import styled from '@emotion/styled'
import type { DeviceInfo, PeakLevels } from '../types'

const Container = styled.div`
  background: #0f3460;
  padding: 20px;
  border-radius: 8px;
`

const Title = styled.h2`
  margin: 0 0 15px 0;
  color: #eee;
`

const MeterGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 15px;
`

const MeterItem = styled.div`
  background: #1a1a2e;
  padding: 15px;
  border-radius: 6px;
`

const MeterLabel = styled.div`
  font-weight: bold;
  margin-bottom: 10px;
  color: #eee;
`

const MeterBar = styled.div<{ level: number }>`
  height: 20px;
  background: linear-gradient(
    to right,
    #00ff00 0%,
    #00ff00 ${props => Math.min(props.level * 100, 60)}%,
    #ffff00 ${props => Math.min(props.level * 100, 80)}%,
    #ff0000 ${props => Math.min(props.level * 100, 100)}%
  );
  border-radius: 4px;
  transition: background 0.1s ease;
`

interface Props {
  levels: PeakLevels
  devices: DeviceInfo[]
}

export default function VUMeter({ levels, devices }: Props) {
  return (
    <Container>
      <Title>峰值电平</Title>
      <MeterGrid>
        {devices.map(device => {
          const level = levels[device.id] || 0
          return (
            <MeterItem key={device.id}>
              <MeterLabel>{device.name}</MeterLabel>
              <MeterBar level={level} />
              <div>{(level * 100).toFixed(1)} dB</div>
            </MeterItem>
          )
        })}
      </MeterGrid>
    </Container>
  )
}
