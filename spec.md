# Otolith Player

## Requirements

- Gapless playback (backend queues up samples for next track well in advance)
- Crossfade
- Equalizer that displays spectrum intensity like FL Studio's Parametric EQ (shows intensity on top of the EQ as you play)
- Cross-platform PC, Mac, Linux
- Able to add folders, google drive, dropbox, etc. and all are unified into a single interface
  - Can deduplicate
- Can edit/manage tags
- Stores playback counts, ratings, last listened times, etc.
  - Can report on playback history, etc.
- Can download lyrics (may be tricky)
- last.fm integration
- Can download metadata and artwork
- Has dark/light themes *and* is able to do reactive on top of that
- Adaptive volume
- Can choose output device
- Has a temp playback queue like Spotify
- When playback queue is over can optionally loop or go shuffle or w/e
  - Dream feature: Vectorize music by style and pick similar???
- Doesn't look like shit :3
- Music library browsing etc.


## Architecture

- Rust backend
  - Realtime
    - Deals with frame-by-frame audio output, acquiring audio device, dealing with loss of and regain of audio device, automatic volume adjustment, muting/volume change (smooth fade), crossfade, prebuffering, EQ, freq spectrum calculations, etc.
    - Runs in separate thread(s); typically there is an OS-owned high perf audio thread that calls back into our code and then there would be another native Rust thread that would talk to the other parts of the application, and these two would work through mutexes etc.
    - Resampling may be required if the audio output device does not support the same sample rate as the music (and if two songs have different sample rates, crossfading and gapless are nearly impossible without building a resampling pipeline. you can really only change the sample rate by reacquiring the output device)
    - Enables the user to "scrub": move within an audio buffer and kinda "hear it as it goes"
  - Loader
    - Asynchronously load audio files
    - Deal with *partial* loads of large files
    - Enumerating files from file sources
    - Can identify wrong file types (.mp3 saved as .flac or w/e)
    - Can read and write metadata
    - Deal with network loads from google drive, dropbox, etc. and with authentication and auth management for these
    - Deal with errors in file loading (not present, network too slow to load in time, etc.)
    - Cache metadata about files somewhere on the user's computer so that if we are loading them e.g. from a network source we can populate the music library without re-ingesting everything
      - Maybe we can have some option to "re-import" a source?
      - We can definitely rewalk local directories at load time, it's fast, but networked... eugh
      - How do we signal to the user that networked sources require manual reload? User education is very tricky here
      - Could have big dialog box that says, "Have you made changes to your &lt;networked source&gt; in the last 13 days?"
    - Also realize that all of these processes of loading are async by necessity. So user is going to see the music library sloooowly populating. How can we make this less ugly?
    - Hydrate album art images for the frontend
  - Logger
    - Store playback information (duration, source, etc.) to local file so that we can show reports etc.
    - All metadata NOT stored in actual files should be computable at program open from the logs
    - sqlite3?  OR: flat file that we can ingest and hydrate
  - Network
    - All social shit (last.fm)
    - All metadata search and download
    - Anything involving lyrics
    - ... yt-dlp integration????

- TS frontend
  - Data ingest
    - Call into the backend *and listen to events from backend* to get an always up-to-date copy of the current state of the music library
    - Same for the status of the realtime system, the queue, etc.
  - UI niceties
    - User can scrub playback bar
    - User can edit queue like a playlist
    - User can see playback history (queue goes into past)
    - User views media library by major screens, where each page is an artist, an album, a playlist, a queue (which is just a playlist), a whole fucking media library, a source, whatever. Like you pick one organizing factor and that determines what you see
    - Sidebar gives you quick access to most of the screens you might want
    - Mini mode
    - Vibe mode (minimal, mostly visualization; a cool-ass now playing that you can fullscreen if you want. Riced up)
  - Theming
    - Dark and light mode
    - Adaptive mode (can change the color scheme within dark/light by analyzing album art)
    - Or people can design their own themes
    - It would be really nice to let users choose their own spacing, fonts, etc. Just to let the user craft a really custom feeling experience
