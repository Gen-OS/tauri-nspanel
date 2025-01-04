interface Props {
  onWindowCreate: () => void
}

export const FloatingIcon = ({ onWindowCreate }: Props) => {
  return (
    <div 
      className="w-32 h-32 cursor-pointer flex items-center justify-center"
      onClick={onWindowCreate}
      style={{ 
        display: 'flex',
        justifyContent: 'center',
      }}
    >
      <img data-tauri-drag-region
        src="/logo.svg" 
        alt="Logo"
        width="128"
        height="128"
        className="select-none pointer-events-none"
        draggable="false"
      />
    </div>
  )
}

export default FloatingIcon