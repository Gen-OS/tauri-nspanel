import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Outlet } from 'react-router-dom'
import FloatingIcon from './components/FloatingIcon'

function App() {
  const [windowCount, setWindowCount] = useState(0)

  const createNewWindow = async () => {
    const options = {
      title: `overlay-${windowCount + 1}`,
      width: 300,
      height: 200,
      x: 100 + (windowCount * 20),
      y: 100 + (windowCount * 20)
    }

    try {
      await invoke('create_overlay_window', { options })
      setWindowCount(prev => prev + 1)
    } catch (e) {
      console.error('Failed to create window:', e)
    }
  }

  return (
    <div data-tauri-drag-region className="w-screen h-screen bg-transparent flex flex-col items-center justify-center pointer-events-auto">
      <FloatingIcon onWindowCreate={createNewWindow} />
      <div data-tauri-drag-region
        className="mt-4 w-32 bg-gray-900 px-4 py-2 rounded-lg shadow-lg border border-gray-700" 
      >
        <span data-tauri-drag-region className="text-white text-sm font-medium select-none">
          Windows: {windowCount}
        </span>
      </div>
      <Outlet />
    </div>
  )
}

export default App