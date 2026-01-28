export interface DeviceInfo {
  id: string
  name: string
  is_input: boolean
  is_output: boolean
  sample_rate: number
  channels: number
  is_vb_cable: boolean
}

export interface Route {
  input_device_id: string
  output_device_id: string
  gain_db: number
  enabled: boolean
}

export type PeakLevels = Record<string, number>
