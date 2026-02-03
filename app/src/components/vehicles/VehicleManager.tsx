/**
 * Vehicle profile manager component
 */

import { useState, useEffect } from "react"
import { useDatabase, type Vehicle, type NewVehicle } from "@/hooks/useDatabase"
import {
  Card,
  Button,
  Badge,
  Alert,
  Modal,
  ConfirmModal,
  EmptyState,
  Skeleton,
} from "@/components/ui"
import {
  Car,
  Plus,
  Edit,
  Trash2,
  Save,
  X,
  Calendar,
  Hash,
  Gauge,
  FileText,
  Search,
} from "lucide-react"

interface VehicleManagerProps {
  onSelect?: (vehicle: Vehicle) => void
  selectedVehicleId?: number | null
}

export function VehicleManager({ onSelect, selectedVehicleId }: VehicleManagerProps) {
  const db = useDatabase()
  const [vehicles, setVehicles] = useState<Vehicle[]>([])
  const [isFormOpen, setIsFormOpen] = useState(false)
  const [editingVehicle, setEditingVehicle] = useState<Vehicle | null>(null)
  const [deleteConfirm, setDeleteConfirm] = useState<Vehicle | null>(null)
  const [searchQuery, setSearchQuery] = useState("")
  const [isLoadingList, setIsLoadingList] = useState(true)

  // Form state
  const [formData, setFormData] = useState<NewVehicle>({
    vin: null,
    make: "BMW",
    model: "",
    year: new Date().getFullYear(),
    engine_code: null,
    mileage_km: null,
    notes: null,
  })

  // Load vehicles
  useEffect(() => {
    loadVehicles()
  }, [])

  const loadVehicles = async () => {
    setIsLoadingList(true)
    try {
      const data = await db.getVehicles()
      setVehicles(data)
    } catch {
      // Error handled in hook
    } finally {
      setIsLoadingList(false)
    }
  }

  const handleSubmit = async () => {
    try {
      if (editingVehicle) {
        await db.updateVehicle(editingVehicle.id, formData)
      } else {
        await db.createVehicle(formData)
      }
      await loadVehicles()
      closeForm()
    } catch {
      // Error handled in hook
    }
  }

  const handleDelete = async () => {
    if (!deleteConfirm) return
    try {
      await db.deleteVehicle(deleteConfirm.id)
      await loadVehicles()
      setDeleteConfirm(null)
    } catch {
      // Error handled in hook
    }
  }

  const openCreateForm = () => {
    setEditingVehicle(null)
    setFormData({
      vin: null,
      make: "BMW",
      model: "",
      year: new Date().getFullYear(),
      engine_code: null,
      mileage_km: null,
      notes: null,
    })
    setIsFormOpen(true)
  }

  const openEditForm = (vehicle: Vehicle) => {
    setEditingVehicle(vehicle)
    setFormData({
      vin: vehicle.vin,
      make: vehicle.make,
      model: vehicle.model,
      year: vehicle.year,
      engine_code: vehicle.engine_code,
      mileage_km: vehicle.mileage_km,
      notes: vehicle.notes,
    })
    setIsFormOpen(true)
  }

  const closeForm = () => {
    setIsFormOpen(false)
    setEditingVehicle(null)
  }

  // Filter vehicles
  const filteredVehicles = vehicles.filter((v) => {
    if (!searchQuery) return true
    const query = searchQuery.toLowerCase()
    return (
      v.model.toLowerCase().includes(query) ||
      v.vin?.toLowerCase().includes(query) ||
      v.engine_code?.toLowerCase().includes(query) ||
      v.year.toString().includes(query)
    )
  })

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex flex-col sm:flex-row gap-3 justify-between">
        <div className="relative flex-1 max-w-xs">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-zinc-500" />
          <input
            type="text"
            placeholder="Search vehicles..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-9 pr-4 py-2 bg-zinc-900 border border-zinc-800 rounded-lg text-sm text-white placeholder-zinc-500 focus:outline-none focus:border-blue-600"
          />
        </div>
        <Button onClick={openCreateForm} leftIcon={<Plus className="h-4 w-4" />}>
          Add Vehicle
        </Button>
      </div>

      {/* Error */}
      {db.error && (
        <Alert variant="error" title="Error" onClose={db.clearError}>
          {db.error}
        </Alert>
      )}

      {/* Vehicle List */}
      {isLoadingList ? (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <Skeleton key={i} className="h-24 w-full" />
          ))}
        </div>
      ) : filteredVehicles.length === 0 ? (
        <Card variant="default" padding="lg">
          <EmptyState
            variant="empty"
            icon={Car}
            title={vehicles.length === 0 ? "No Vehicles" : "No Results"}
            description={
              vehicles.length === 0
                ? "Add your first vehicle to start tracking diagnostics"
                : "No vehicles match your search"
            }
            action={
              vehicles.length === 0
                ? { label: "Add Vehicle", onClick: openCreateForm }
                : undefined
            }
          />
        </Card>
      ) : (
        <div className="space-y-3">
          {filteredVehicles.map((vehicle) => (
            <VehicleCard
              key={vehicle.id}
              vehicle={vehicle}
              isSelected={selectedVehicleId === vehicle.id}
              onSelect={() => onSelect?.(vehicle)}
              onEdit={() => openEditForm(vehicle)}
              onDelete={() => setDeleteConfirm(vehicle)}
            />
          ))}
        </div>
      )}

      {/* Create/Edit Modal */}
      <Modal
        isOpen={isFormOpen}
        onClose={closeForm}
        title={editingVehicle ? "Edit Vehicle" : "Add Vehicle"}
        size="lg"
      >
        <form
          onSubmit={(e) => {
            e.preventDefault()
            handleSubmit()
          }}
          className="space-y-4"
        >
          <div className="grid grid-cols-2 gap-4">
            {/* Make */}
            <div>
              <label className="block text-sm text-zinc-400 mb-1">Make</label>
              <input
                type="text"
                value={formData.make}
                onChange={(e) => setFormData({ ...formData, make: e.target.value })}
                className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white focus:outline-none focus:border-blue-600"
                required
              />
            </div>

            {/* Model */}
            <div>
              <label className="block text-sm text-zinc-400 mb-1">Model</label>
              <input
                type="text"
                value={formData.model}
                onChange={(e) => setFormData({ ...formData, model: e.target.value })}
                placeholder="e.g., 520d E60"
                className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white focus:outline-none focus:border-blue-600"
                required
              />
            </div>

            {/* Year */}
            <div>
              <label className="block text-sm text-zinc-400 mb-1">Year</label>
              <input
                type="number"
                value={formData.year}
                onChange={(e) => setFormData({ ...formData, year: parseInt(e.target.value) || 0 })}
                min={1990}
                max={new Date().getFullYear() + 1}
                className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white focus:outline-none focus:border-blue-600"
                required
              />
            </div>

            {/* Engine Code */}
            <div>
              <label className="block text-sm text-zinc-400 mb-1">Engine Code</label>
              <input
                type="text"
                value={formData.engine_code || ""}
                onChange={(e) => setFormData({ ...formData, engine_code: e.target.value || null })}
                placeholder="e.g., M47TU2D20"
                className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white focus:outline-none focus:border-blue-600"
              />
            </div>

            {/* VIN */}
            <div className="col-span-2">
              <label className="block text-sm text-zinc-400 mb-1">VIN</label>
              <input
                type="text"
                value={formData.vin || ""}
                onChange={(e) => setFormData({ ...formData, vin: e.target.value.toUpperCase() || null })}
                placeholder="17-character VIN"
                maxLength={17}
                className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white font-mono focus:outline-none focus:border-blue-600"
              />
            </div>

            {/* Mileage */}
            <div>
              <label className="block text-sm text-zinc-400 mb-1">Mileage (km)</label>
              <input
                type="number"
                value={formData.mileage_km || ""}
                onChange={(e) => setFormData({ ...formData, mileage_km: parseInt(e.target.value) || null })}
                placeholder="Current mileage"
                min={0}
                className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white focus:outline-none focus:border-blue-600"
              />
            </div>

            {/* Notes */}
            <div className="col-span-2">
              <label className="block text-sm text-zinc-400 mb-1">Notes</label>
              <textarea
                value={formData.notes || ""}
                onChange={(e) => setFormData({ ...formData, notes: e.target.value || null })}
                placeholder="Optional notes about this vehicle"
                rows={3}
                className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white resize-none focus:outline-none focus:border-blue-600"
              />
            </div>
          </div>

          <div className="flex justify-end gap-3 pt-4 border-t border-zinc-800">
            <Button type="button" variant="ghost" onClick={closeForm}>
              <X className="h-4 w-4" />
              Cancel
            </Button>
            <Button type="submit" loading={db.isLoading} leftIcon={<Save className="h-4 w-4" />}>
              {editingVehicle ? "Save Changes" : "Add Vehicle"}
            </Button>
          </div>
        </form>
      </Modal>

      {/* Delete Confirmation */}
      <ConfirmModal
        isOpen={!!deleteConfirm}
        onClose={() => setDeleteConfirm(null)}
        onConfirm={handleDelete}
        title="Delete Vehicle"
        message={`Are you sure you want to delete "${deleteConfirm?.make} ${deleteConfirm?.model}"? This will also delete all associated diagnostic sessions and DTCs.`}
        confirmText="Delete"
        variant="danger"
        loading={db.isLoading}
      />
    </div>
  )
}

// Vehicle card component
interface VehicleCardProps {
  vehicle: Vehicle
  isSelected: boolean
  onSelect: () => void
  onEdit: () => void
  onDelete: () => void
}

function VehicleCard({ vehicle, isSelected, onSelect, onEdit, onDelete }: VehicleCardProps) {
  return (
    <Card
      variant={isSelected ? "interactive" : "elevated"}
      padding="md"
      className={`cursor-pointer transition-all ${
        isSelected ? "border-blue-600 shadow-[0_0_20px_rgba(0,102,177,0.2)]" : ""
      }`}
      onClick={onSelect}
    >
      <div className="flex items-start justify-between">
        <div className="flex items-start gap-3">
          <div className="p-2 rounded-lg bg-blue-600/20 border border-blue-600/30">
            <Car className="h-5 w-5 text-blue-400" />
          </div>
          <div>
            <h3 className="font-semibold text-white">
              {vehicle.make} {vehicle.model}
            </h3>
            <div className="flex flex-wrap gap-2 mt-1">
              <Badge variant="outline" size="sm">
                <Calendar className="h-3 w-3" />
                {vehicle.year}
              </Badge>
              {vehicle.engine_code && (
                <Badge variant="outline" size="sm">
                  <Hash className="h-3 w-3" />
                  {vehicle.engine_code}
                </Badge>
              )}
              {vehicle.mileage_km && (
                <Badge variant="outline" size="sm">
                  <Gauge className="h-3 w-3" />
                  {vehicle.mileage_km.toLocaleString()} km
                </Badge>
              )}
            </div>
            {vehicle.vin && (
              <p className="text-xs font-mono text-zinc-500 mt-2">
                VIN: {vehicle.vin}
              </p>
            )}
          </div>
        </div>
        <div className="flex gap-1" onClick={(e) => e.stopPropagation()}>
          <Button size="icon-sm" variant="ghost" onClick={onEdit}>
            <Edit className="h-4 w-4" />
          </Button>
          <Button size="icon-sm" variant="ghost" onClick={onDelete} className="text-red-400 hover:text-red-300">
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </div>
      {vehicle.notes && (
        <div className="mt-3 pt-3 border-t border-zinc-800 flex items-start gap-2 text-sm text-zinc-400">
          <FileText className="h-4 w-4 shrink-0 mt-0.5" />
          <p className="truncate-2">{vehicle.notes}</p>
        </div>
      )}
    </Card>
  )
}
