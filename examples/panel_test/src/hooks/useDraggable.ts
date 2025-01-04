import { useState, useCallback, useEffect } from 'react'

interface Position {
  x: number
  y: number
}

interface UseDraggableProps {
  onDragStart?: () => void
  onDragEnd?: () => void
  initialPosition?: Position
}

export const useDraggable = ({ 
  onDragStart, 
  onDragEnd,
  initialPosition = { x: 0, y: 0 } 
}: UseDraggableProps = {}) => {
  const [isDragging, setIsDragging] = useState(false)
  const [position, setPosition] = useState<Position>(initialPosition)
  const [dragStart, setDragStart] = useState<Position>({ x: 0, y: 0 })
  const [elementOffset, setElementOffset] = useState<Position>({ x: 0, y: 0 })

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    const element = e.currentTarget as HTMLElement
    const rect = element.getBoundingClientRect()
    
    setElementOffset({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top
    })
    
    setDragStart({
      x: e.clientX,
      y: e.clientY
    })
    
    setIsDragging(true)
    onDragStart?.()
  }, [onDragStart])

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isDragging) return

    setPosition({
      x: e.clientX - elementOffset.x,
      y: e.clientY - elementOffset.y
    })
  }, [isDragging, elementOffset])

  const handleMouseUp = useCallback(() => {
    if (isDragging) {
      setIsDragging(false)
      onDragEnd?.()
    }
  }, [isDragging, onDragEnd])

  useEffect(() => {
    if (isDragging) {
      window.addEventListener('mousemove', handleMouseMove)
      window.addEventListener('mouseup', handleMouseUp)
    }

    return () => {
      window.removeEventListener('mousemove', handleMouseMove)
      window.removeEventListener('mouseup', handleMouseUp)
    }
  }, [isDragging, handleMouseMove, handleMouseUp])

  return {
    isDragging,
    position,
    handleMouseDown,
    dragStart
  }
}