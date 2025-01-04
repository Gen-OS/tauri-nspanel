import { createBrowserRouter } from 'react-router-dom'
import App from './App'
import OverlayWindow from './components/OverlayWindow'

export const router = createBrowserRouter([
  {
    path: '/',
    element: <App />
  },
  {
    path: '/overlay',
    element: <OverlayWindow />
  }
])