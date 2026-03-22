# resolution-news-aggregator

A rust program to aggregate multiple RSS source to websocket to be broadcast to multiple client, and display it on a website!

TODO:
- Server
  - [x] Read config
  - [x] Spawn individual task for each config kind
  - [x] Config file
  - [ ] Google News URL resolve
  - [ ] SQL database
- RSS Source
  - [ ] Google News Domain based 
  - [x] Hacker News
  - [x] Youtube
- UI
  - [ ] Multi-window
  - [ ] Content filtering (Filter the entire window and stored)
  - [ ] Search feature