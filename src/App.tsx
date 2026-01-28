import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import styled from '@emotion/styled'
import DeviceList from './components/DeviceList'
import RoutePanel from './components/RoutePanel'
import VUMeter from './components/VUMeter'
import type { DeviceInfo, Route } from './types'

const AppContainer = styled.div`
  display: flex;
  height: 100vh;
  background: #1a1a2e;
  color: #eee;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
`

const Sidebar = styled.div`
  width: 300px;
  background: #16213e;
  padding: 20px;
  border-right: 1px solid #0f3460;
`

const MainContent = styled.div`
  flex: 1;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
`

const Header = styled.h1`
  margin: 0 0 20px 0;
  color: #e94560;
`

export default function App() {
  const [devices, setDevices] = useState<DeviceInfo[]>([])
  const [routes, setRoutes] = useState<Route[]>([])
  const [peakLevels, setPeakLevels] = useState<Record<string, number>>({})
  const [isRunning, setIsRunning] = useState(false)

  useEffect(() => {
    loadDevices()
    const interval = setInterval(updatePeakLevels, 100)
    return () => clearInterval(interval)
  }, [])

  const loadDevices = async () => {
    try {
      const deviceList = await invoke<DeviceInfo[]>('list_devices')
      setDevices(deviceList)
    } catch (error) {
      console.error('Failed to load devices:', error)
    }
  }

  const updatePeakLevels = async () => {
    try {
      const peaks = await invoke<Record<string, number>>('get_peak_levels')
      setPeakLevels(peaks)
    } catch (error) {
      console.error('Failed to update peak levels:', error)
    }
  }

  const handleStart = async () => {
    try {
      await invoke('start_engine')
      setIsRunning(true)
    } catch (error) {
      console.error('Failed to start engine:', error)
    }
  }

  const handleStop = async () => {
    try {
      await invoke('stop_engine')
      setIsRunning(false)
    } catch (error) {
      console.error('Failed to stop engine:', error)
    }
  }

  return (
    <AppContainer>
      <Sidebar>
        <Header>Audio Flow</Header>
        <button onClick={loadDevices}>刷新设备</button>
        <button onClick={isRunning ? handleStop : handleStart}>
          {isRunning ? '停止' : '启动'}
        </button>
        <DeviceList devices={devices} />
      </Sidebar>
      
      <MainContent>
        <RoutePanel 
          devices={devices}
          routes={routes}
          onRoutesChange={setRoutes}
        />
        
        <VUMeter 
          levels={peakLevels}
          devices={devices}
        />
      </MainContent>
    </AppContainer>
  )
}
