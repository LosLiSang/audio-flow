import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import styled from '@emotion/styled'
import type { DeviceInfo, Route } from '../types'

const Container = styled.div`
  background: #0f3460;
  padding: 20px;
  border-radius: 8px;
`

const Title = styled.h2`
  margin: 0 0 15px 0;
  color: #eee;
`

const RouteList = styled.div`
  display: flex;
  flex-direction: column;
  gap: 10px;
`

const RouteItem = styled.div`
  display: flex;
  align-items: center;
  gap: 15px;
  padding: 15px;
  background: #1a1a2e;
  border-radius: 6px;
`

const RouteInfo = styled.div`
  flex: 1;
`

const RouteName = styled.div`
  font-weight: bold;
  margin-bottom: 5px;
`

const GainSlider = styled.input`
  width: 150px;
`

const RemoveButton = styled.button`
  background: #e94560;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;

  &:hover {
    background: #d63850;
  }
`

const DeviceSelect = styled.div`
  display: flex;
  gap: 15px;
  margin-bottom: 15px;
  align-items: center;

  select {
    flex: 1;
    padding: 10px;
    background: #1a1a2e;
    color: #eee;
    border: 1px solid #0f3460;
    border-radius: 4px;
    font-size: 14px;

    &:focus {
      outline: none;
      border-color: #e94560;
    }
  }
`

const ErrorMessage = styled.div`
  color: #e94560;
  background: rgba(233, 69, 96, 0.1);
  padding: 10px;
  border-radius: 4px;
  margin-bottom: 15px;
  font-size: 14px;
`

interface Props {
  devices: DeviceInfo[]
  routes: Route[]
  onRoutesChange: (routes: Route[]) => void
  onRoutesReload?: () => Promise<void>
}

export default function RoutePanel({ devices, routes, onRoutesChange, onRoutesReload }: Props) {
  const [adding, setAdding] = useState(false)
  const [selectedInputId, setSelectedInputId] = useState<string>('')
  const [selectedOutputId, setSelectedOutputId] = useState<string>('')
  const [error, setError] = useState<string | null>(null)

  const inputDevices = devices.filter(d => d.is_input)
  const outputDevices = devices.filter(d => d.is_output)

  const handleAddRoute = async () => {
    console.log('handleAddRoute called', { selectedInputId, selectedOutputId, devices })
    if (!selectedInputId || !selectedOutputId) {
      setError('请选择输入和输出设备')
      console.log('No device selected')
      return
    }

    const inputDevice = devices.find(d => d.id === selectedInputId)
    const outputDevice = devices.find(d => d.id === selectedOutputId)
    
    console.log('Found devices:', { inputDevice, outputDevice })
    
    if (inputDevice && outputDevice) {
      const newRoute: Route = {
        input_device_id: inputDevice.id,
        output_device_id: outputDevice.id,
        gain_db: 0,
        enabled: true,
      }
      
      console.log('Adding route:', newRoute)
      
      try {
        setAdding(true)
        setError(null)
        const result = await invoke('add_route', { route: newRoute })
        console.log('add_route result:', result)
        // 更新本地状态
        onRoutesChange([...routes, newRoute])
        console.log('Local routes updated, count:', routes.length + 1)
        // 重新从后端加载确保一致性
        if (onRoutesReload) {
          await onRoutesReload()
          console.log('Routes reloaded from backend')
        }
        // 重置选择
        setSelectedInputId('')
        setSelectedOutputId('')
      } catch (error) {
        console.error('Failed to add route:', error)
        setError('添加路由失败：' + (error as Error).message)
      } finally {
        setAdding(false)
      }
    } else {
      setError('选择的设备不存在')
      console.log('Devices not found')
    }
  }

  const handleRemoveRoute = async (inputId: string, outputId: string) => {
    console.log('Removing route:', { inputId, outputId })
    try {
      const result = await invoke('remove_route', { inputId, outputId })
      console.log('remove_route result:', result)
      // 重新从后端加载确保一致性
      if (onRoutesReload) {
        await onRoutesReload()
        console.log('Routes reloaded after removal')
      }
    } catch (error) {
      console.error('Failed to remove route:', error)
      const errorMessage = error instanceof Error ? error.message : String(error)
      setError('删除路由失败：' + errorMessage)
    }
  }

  const handleGainChange = async (deviceId: string, gain: number) => {
    console.log('Setting gain:', { deviceId, gain })
    try {
      const result = await invoke('set_gain', { deviceId, gainDb: gain })
      console.log('set_gain result:', result)
      const newRoutes = [...routes]
      const routeIndex = newRoutes.findIndex(r => r.input_device_id === deviceId)
      if (routeIndex !== -1) {
        newRoutes[routeIndex].gain_db = gain
        onRoutesChange(newRoutes)
      }
      // 增益改变不需要重新加载整个路由列表
    } catch (error) {
      console.error('Failed to set gain:', error)
      const errorMessage = error instanceof Error ? error.message : String(error)
      setError('设置增益失败：' + errorMessage)
    }
  }

  return (
    <Container>
      <Title>音频路由</Title>
      
      {error && <ErrorMessage>{error}</ErrorMessage>}
      
      <DeviceSelect>
        <select value={selectedInputId} onChange={(e) => setSelectedInputId(e.target.value)}>
          <option value="">-- 选择输入设备 --</option>
          {inputDevices.map(device => (
            <option key={device.id} value={device.id}>
              {device.name} {device.is_vb_cable && '(VB-Cable)'}
            </option>
          ))}
        </select>
        
        <select value={selectedOutputId} onChange={(e) => setSelectedOutputId(e.target.value)}>
          <option value="">-- 选择输出设备 --</option>
          {outputDevices.map(device => (
            <option key={device.id} value={device.id}>
              {device.name} {device.is_vb_cable && '(VB-Cable)'}
            </option>
          ))}
        </select>
      </DeviceSelect>
      
      <button onClick={handleAddRoute} disabled={adding}>
        {adding ? '添加中...' : '添加路由'}
      </button>
      <RouteList>
        {routes.map((route) => {
          const inputDevice = devices.find(d => d.id === route.input_device_id)
          const outputDevice = devices.find(d => d.id === route.output_device_id)
          const routeKey = `${route.input_device_id}-${route.output_device_id}`
          
          return (
            <RouteItem key={routeKey}>
              <RouteInfo>
                <RouteName>
                  {inputDevice?.name || 'Unknown'} → {outputDevice?.name || 'Unknown'}
                </RouteName>
              </RouteInfo>
              <GainSlider
                type="range"
                min="-60"
                max="12"
                step="1"
                value={route.gain_db}
                onChange={(e) => handleGainChange(route.input_device_id, parseFloat(e.target.value))}
              />
              <span>{route.gain_db} dB</span>
              <RemoveButton onClick={() => handleRemoveRoute(route.input_device_id, route.output_device_id)}>移除</RemoveButton>
            </RouteItem>
          )
        })}
      </RouteList>
    </Container>
  )
}
