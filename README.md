# Jiji

A simple discord client aimed at lightness, not as feature complete as the official one, but the goal is to have this in the background (taking few resources) while chatting. When you want to do some proper activities on discord, you can use the heavier, more feature complete official client

# GUI
using egui to have immediate mod rendering (so the app is using very few processing power when not interacted with)  
  

# Discord Backend
using Serenity, with a discord bot (you need to have a token)

# Screeshots
![screenshot](./assets/screenshot.png)

# Features
- messages read/write in channels or direct messages  
- direct messages : initiated by other users, but persistent  
- no voice as we are using a bot  
- notifications : disabled by default, you enable them channel by channel  

# Need further testing

maybe able to use user token instead of bot token, but careful, you might get banned  
-> may be a solution to access voice features (need some more research)