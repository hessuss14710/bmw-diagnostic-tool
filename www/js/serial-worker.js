/**
 * Serial Worker - Handles timing-critical serial communication
 * Runs in separate thread to avoid main thread blocking
 */

// Ring buffer for incoming data (more efficient than array splice)
class RingBuffer {
    constructor(size = 4096) {
        this.buffer = new Uint8Array(size);
        this.head = 0;
        this.tail = 0;
        this.size = size;
    }

    push(data) {
        for (const byte of data) {
            this.buffer[this.head] = byte;
            this.head = (this.head + 1) % this.size;
            if (this.head === this.tail) {
                // Buffer overflow - move tail
                this.tail = (this.tail + 1) % this.size;
            }
        }
    }

    available() {
        if (this.head >= this.tail) {
            return this.head - this.tail;
        }
        return this.size - this.tail + this.head;
    }

    read(length) {
        const available = this.available();
        const toRead = Math.min(length, available);
        const result = new Uint8Array(toRead);

        for (let i = 0; i < toRead; i++) {
            result[i] = this.buffer[this.tail];
            this.tail = (this.tail + 1) % this.size;
        }

        return result;
    }

    peek(length) {
        const available = this.available();
        const toRead = Math.min(length, available);
        const result = new Uint8Array(toRead);

        let pos = this.tail;
        for (let i = 0; i < toRead; i++) {
            result[i] = this.buffer[pos];
            pos = (pos + 1) % this.size;
        }

        return result;
    }

    clear() {
        this.head = 0;
        this.tail = 0;
    }
}

// Message queue with priorities
class PriorityQueue {
    constructor() {
        this.high = [];    // TesterPresent, critical commands
        this.normal = [];  // DTCs, standard commands
        this.low = [];     // Live data polling
    }

    enqueue(message, priority = 'normal') {
        const queue = this[priority] || this.normal;
        queue.push(message);
    }

    dequeue() {
        if (this.high.length > 0) return this.high.shift();
        if (this.normal.length > 0) return this.normal.shift();
        if (this.low.length > 0) return this.low.shift();
        return null;
    }

    isEmpty() {
        return this.high.length === 0 &&
               this.normal.length === 0 &&
               this.low.length === 0;
    }

    clear() {
        this.high = [];
        this.normal = [];
        this.low = [];
    }
}

// KWP2000 Message Builder
function buildKWPMessage(source, target, data) {
    const length = data.length;
    const fmt = 0x80 | length;
    const message = [fmt, target, source, ...data];
    const checksum = message.reduce((a, b) => a + b, 0) & 0xFF;
    message.push(checksum);
    return new Uint8Array(message);
}

// KWP2000 Response Parser
function parseKWPResponse(buffer, data) {
    if (data.length < 4) return null;

    const fmt = data[0];
    const target = data[1];
    const source = data[2];

    let dataLength, dataStart;

    if ((fmt & 0x80) !== 0) {
        // Length in format byte
        dataLength = fmt & 0x3F;
        dataStart = 3;
    } else if (fmt === 0x80) {
        // Length in separate byte
        if (data.length < 5) return null;
        dataLength = data[3];
        dataStart = 4;
    } else {
        return null;
    }

    const totalLength = dataStart + dataLength + 1; // +1 for checksum

    if (data.length < totalLength) return null;

    // Verify checksum
    let calcChecksum = 0;
    for (let i = 0; i < totalLength - 1; i++) {
        calcChecksum = (calcChecksum + data[i]) & 0xFF;
    }

    if (calcChecksum !== data[totalLength - 1]) {
        return { error: 'checksum', expected: calcChecksum, received: data[totalLength - 1] };
    }

    const responseData = data.slice(dataStart, dataStart + dataLength);
    const service = responseData[0];

    return {
        source,
        target,
        service,
        data: responseData,
        length: totalLength
    };
}

// Frame synchronization - find valid KWP message start
function findFrameStart(data) {
    for (let i = 0; i < data.length - 3; i++) {
        const fmt = data[i];
        // Valid format byte: 0x80-0xBF (length in format) or 0x80 (length separate)
        if ((fmt & 0xC0) === 0x80 || fmt === 0x80) {
            // Could be valid frame start
            return i;
        }
    }
    return -1;
}

// High-resolution timing using performance.now()
function preciseDelay(ms) {
    return new Promise(resolve => {
        const start = performance.now();
        const check = () => {
            if (performance.now() - start >= ms) {
                resolve();
            } else {
                // Use immediate callback for <10ms, otherwise setTimeout
                if (ms - (performance.now() - start) < 10) {
                    setImmediate ? setImmediate(check) : setTimeout(check, 0);
                } else {
                    setTimeout(check, 1);
                }
            }
        };
        check();
    });
}

// Worker state
let rxBuffer = new RingBuffer(4096);
let txQueue = new PriorityQueue();
let isProcessing = false;
let ecuAddress = null;
let lastTesterPresent = 0;

// Message handlers
self.onmessage = async function(e) {
    const { type, data } = e.data;

    switch (type) {
        case 'rx_data':
            // Data received from serial port
            rxBuffer.push(new Uint8Array(data));
            processRxBuffer();
            break;

        case 'send':
            // Queue message for transmission
            txQueue.enqueue({
                data: data.bytes,
                callback: data.id,
                timeout: data.timeout || 2000
            }, data.priority || 'normal');
            processTxQueue();
            break;

        case 'set_ecu':
            ecuAddress = data.address;
            break;

        case 'clear_buffer':
            rxBuffer.clear();
            break;

        case 'clear_queue':
            txQueue.clear();
            break;

        case 'tester_present':
            // High priority TesterPresent
            if (ecuAddress && Date.now() - lastTesterPresent > 1500) {
                const msg = buildKWPMessage(0xF1, ecuAddress, [0x3E]);
                txQueue.enqueue({
                    data: Array.from(msg),
                    callback: 'tester_present',
                    timeout: 500,
                    silent: true
                }, 'high');
                lastTesterPresent = Date.now();
                processTxQueue();
            }
            break;

        case 'read_pids':
            // Batch PID read - low priority
            for (const pid of data.pids) {
                const msg = buildKWPMessage(0xF1, ecuAddress, [0x21, pid]);
                txQueue.enqueue({
                    data: Array.from(msg),
                    callback: `pid_${pid}`,
                    timeout: 300,
                    pid: pid
                }, 'low');
            }
            processTxQueue();
            break;
    }
};

// Process received data buffer
function processRxBuffer() {
    while (rxBuffer.available() >= 4) {
        const peek = rxBuffer.peek(50);
        const frameStart = findFrameStart(Array.from(peek));

        if (frameStart > 0) {
            // Discard bytes before frame
            rxBuffer.read(frameStart);
            continue;
        }

        if (frameStart < 0) {
            // No valid frame found, wait for more data
            break;
        }

        // Try to parse message
        const data = Array.from(rxBuffer.peek(50));
        const parsed = parseKWPResponse(rxBuffer, data);

        if (parsed === null) {
            // Incomplete message, wait for more data
            break;
        }

        if (parsed.error) {
            // Invalid checksum, skip first byte and try again
            rxBuffer.read(1);
            self.postMessage({ type: 'error', data: { message: 'Checksum error', details: parsed } });
            continue;
        }

        // Valid message - consume from buffer
        rxBuffer.read(parsed.length);

        // Send to main thread
        self.postMessage({
            type: 'rx_message',
            data: {
                source: parsed.source,
                target: parsed.target,
                service: parsed.service,
                data: Array.from(parsed.data),
                timestamp: performance.now()
            }
        });
    }
}

// Process transmission queue
async function processTxQueue() {
    if (isProcessing) return;
    isProcessing = true;

    while (!txQueue.isEmpty()) {
        const msg = txQueue.dequeue();
        if (!msg) break;

        // Request transmission
        self.postMessage({
            type: 'tx_request',
            data: {
                bytes: msg.data,
                callback: msg.callback,
                timeout: msg.timeout
            }
        });

        // Wait for inter-message delay (P3 timing)
        await preciseDelay(50);
    }

    isProcessing = false;
}

// Periodic TesterPresent (keep-alive)
setInterval(() => {
    if (ecuAddress) {
        self.postMessage({ type: 'request_tester_present' });
    }
}, 2000);

self.postMessage({ type: 'ready' });
