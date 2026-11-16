class Air {
  over = $state<{ id: string; at: number } | null>(null)
  slot: (id: string, x: number, y: number) => number = () => -1
}

export const air = new Air()
