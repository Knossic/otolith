import React from "react";
import {
  Play,
  Pause,
  SkipBack,
  SkipForward,
  Shuffle,
  Repeat,
  Volume2,
  Heart,
  Maximize2,
} from "lucide-react";

const MusicPlayer = () => {
  const [isPlaying, setIsPlaying] = React.useState(false);
  const [isLiked, setIsLiked] = React.useState(false);

  return (
    <div className="h-20 bg-gradient-to-r from-gray-900 to-black border-t border-gray-800 px-4 flex items-center justify-between">
      
      {/* Left Section - Currently Playing */}
      <div className="flex items-center space-x-4 w-1/3">
        <div className="w-14 h-14 bg-gray-700 rounded-md flex items-center justify-center">
          <span className="text-gray-400 text-xs">Album</span>
        </div>
        <div className="min-w-0">
          <h4 className="text-white text-sm font-medium truncate">Song Title</h4>
          <p className="text-gray-400 text-xs truncate">Artist Name</p>
        </div>
        <button
          onClick={() => setIsLiked(!isLiked)}
          className={`p-1 rounded-full transition-colors ${
            isLiked ? "text-green-500" : "text-gray-400 hover:text-white"
          }`}
        >
          <Heart className="w-4 h-4" fill={isLiked ? "currentColor" : "none"} />
        </button>
      </div>

      {/* Center Section - Playback Controls */}
      <div className="flex flex-col items-center space-y-2 w-1/3">
        {/* Control Buttons */}
        <div className="flex items-center space-x-4">
          <button className="text-gray-400 hover:text-white transition-colors">
            <Shuffle className="w-4 h-4" />
          </button>
          <button className="text-gray-400 hover:text-white transition-colors">
            <SkipBack className="w-5 h-5" />
          </button>
          <button
            onClick={() => setIsPlaying(!isPlaying)}
            className="bg-white text-black rounded-full p-2 hover:scale-105 transition-transform"
          >
            {isPlaying ? (
              <Pause className="w-5 h-5" />
            ) : (
              <Play className="w-5 h-5 ml-0.5" />
            )}
          </button>
          <button className="text-gray-400 hover:text-white transition-colors">
            <SkipForward className="w-5 h-5" />
          </button>
          <button className="text-gray-400 hover:text-white transition-colors">
            <Repeat className="w-4 h-4" />
          </button>
        </div>

        {/* Progress Bar */}
        <div className="flex items-center space-x-2 w-full max-w-md">
          <span className="text-xs text-gray-400">1:23</span>
          <div className="flex-1 bg-gray-600 rounded-full h-1">
            <div className="bg-white rounded-full h-1 w-1/3 relative">
              <div className="absolute right-0 top-1/2 transform -translate-y-1/2 w-3 h-3 bg-white rounded-full opacity-0 hover:opacity-100 transition-opacity"></div>
            </div>
          </div>
          <span className="text-xs text-gray-400">3:45</span>
        </div>
      </div>

      {/* Right Section - Volume & Additional Controls */}
      <div className="flex items-center space-x-3 w-1/3 justify-end">
        <button className="text-gray-400 hover:text-white transition-colors">
          <Maximize2 className="w-4 h-4" />
        </button>
        <div className="flex items-center space-x-2">
          <Volume2 className="w-4 h-4 text-gray-400" />
          <div className="w-20 bg-gray-600 rounded-full h-1">
            <div className="bg-white rounded-full h-1 w-3/4 relative">
              <div className="absolute right-0 top-1/2 transform -translate-y-1/2 w-3 h-3 bg-white rounded-full opacity-0 hover:opacity-100 transition-opacity"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default MusicPlayer; 