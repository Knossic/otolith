import React from "react";
import "./App.css";
import MusicPlayer from "./MusicPlayer";

function App() {
  return (
    <div className="h-screen flex flex-col bg-black text-white">
      {/* Main content area */}
      <div className="flex-1">
        {/* Content will go here later */}
      </div>
      
      {/* Bottom bar container */}
      <MusicPlayer />
    </div>
  );
}

export default App;
