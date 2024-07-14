# Jiji

A simple discord client aimed at lightness, not as feature complete as the official one, but the goal is to have this in the background (taking few resources) while chatting. When you want to do some proper activities on discord, you can use the heavier, more feature complete official client

# GUI
using egui to have immediate mod rendering (so the app is using very few processing power when not interacted with)  
  

# Discord Backend
using Serenity, with a discord bot (you need to have a token)

# Screenshot
![screenshot](./assets/screenshot.png)

# Features
- messages read/write in channels or direct messages  
- direct messages : initiated by other users, but persistent  
- no voice as we are using a bot  
- notifications : disabled by default, you enable them channel by channel  

# Performances
best case (startup) / use case (100 messages loaded in current channel, 10 channels, 2 servers)  

ram usage : 105 MB / 109 MB 
  
frame calculation time : 0.3ms / 1ms  with a Ryzen 7 3700U
  
jiji is capped at 30 FPS (max 1 frame every 33.3ms) but 0.5 FPS when not interacted with (1 frame every 2000ms)
  
that way it is very light on the processor as well

# Need further testing

maybe able to use user token instead of bot token, but careful, you might get banned  
-> may be a solution to access voice features (need some more research)