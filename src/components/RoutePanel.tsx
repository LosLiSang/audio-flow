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

interface Props {
  devices: DeviceInfo[]
  routes: Route[]
  onRoutesChange: (routes: Route[]) => void
}

export default function RoutePanel({ devices, routes, onRoutesChange }: Props) {
  const handleAddRoute = () => {
    const inputDevice = devices.find(d => d.is_input)
    const outputDevice = devices.find(d => d.is_output && d.is_vb_cable)
    
    if (inputDevice && outputDevice) {
      const newRoute: Route = {
        input_device_id: inputDevice.id,
        output_device_id: outputDevice.id,
        gain_db: 0,
        enabled: true,
      }
      onRoutesChange([...routes, newRoute])
    }
  }

  const handleRemoveRoute = (index: number) => {
    const newRoutes = routes.filter((_, i) => i !== index)
    onRoutesChange(newRoutes)
  }

  const handleGainChange = (index: number, gain: number) => {
    const newRoutes = [...routes]
    newRoutes[index].gain_db = gain
    onRoutesChange(newRoutes)
  }

  return (
    <Container>
      <Title>音频路由</Title>
      <button onClick={handleAddRoute}>添加路由</button>
      <RouteList>
        {routes.map((route, index) => {
          const inputDevice = devices.find(d => d.id === route.input_device_id)
          const outputDevice = devices.find(d => d.id === route.output_device_id)
          
          return (
            <RouteItem key={index}>
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
                onChange={(e) => handleGainChange(index, parseFloat(e.target.value))}
              />
              <span>{route.gain_db} dB</span>
              <RemoveButton onClick={() => handleRemoveRoute(index)}>移除</RemoveButton>
            </RouteItem>
          )
        })}
      </RouteList>
    </Container>
  )
}
