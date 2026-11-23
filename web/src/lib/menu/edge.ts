const gap = 8

export function edge(x: number, y: number, w: number, h: number, vw: number, vh: number) {
  return {
    x: x + w > vw - gap ? Math.max(gap, x - w) : x,
    y: y + h > vh - gap ? Math.max(gap, y - h) : y,
  }
}
