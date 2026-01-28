import styled from '@emotion/styled'
import type { DeviceInfo } from '../types'

const Container = styled.div`
  margin-top: 20px;
`

const Title = styled.h3`
  margin: 0 0 10px 0;
  color: #eee;
`

const DeviceItem = styled.div<{ is_vb_cable: boolean }>`
  padding: 10px;
  margin-bottom: 8px;
  background: ${props => props.is_vb_cable ? '#0f3460' : '#0a0f24'};
  border-radius: 6px;
  border-left: 3px solid ${props => props.is_vb_cable ? '#e94560' : '#533483'};
`

const DeviceName = styled.div`
  font-weight: bold;
  margin-bottom: 4px;
`

const DeviceInfo = styled.div`
  font-size: 0.85em;
  color: #aaa;
`

interface Props {
  devices: DeviceInfo[]
}

export default function DeviceList({ devices }: Props) {
  const inputDevices = devices.filter(d => d.is_input)
  const outputDevices = devices.filter(d => d.is_output)

  return (
    <Container>
      <Title>输入设备 ({inputDevices.length})</Title>
      {inputDevices.map(device => (
        <DeviceItem key={device.id} is_vb_cable={device.is_vb_cable}>
          <DeviceName>{device.name}</DeviceName>
          <DeviceInfo>
            {device.channels} ch • {device.sample_rate} Hz
            {device.is_vb_cable && ' • VB-Cable'}
          </DeviceInfo>
        </DeviceItem>
      ))}
      
      <Title style={{ marginTop: '20px' }}>输出设备 ({outputDevices.length})</Title>
      {outputDevices.map(device => (
        <DeviceItem key={device.id} is_vb_cable={device.is_vb_cable}>
          <DeviceName>{device.name}</DeviceName>
          <DeviceInfo>
            {device.channels} ch • {device.sample_rate} Hz
            {device.is_vb_cable && ' • VB-Cable'}
          </DeviceInfo>
        </DeviceItem>
      ))}
    </Container>
  )
}
