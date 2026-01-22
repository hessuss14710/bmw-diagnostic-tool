FROM node:22-slim

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY dist ./dist
COPY src/data ./dist/data

ENV NODE_ENV=production

EXPOSE 3002

CMD ["node", "dist/index.js"]
