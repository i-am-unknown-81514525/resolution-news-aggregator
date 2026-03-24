# resolution-news-aggregator

A rust program to aggregate multiple RSS source to websocket to be broadcast to multiple client, and display it on a website!

Example:

<img width="1008" height="1970" alt="image" src="https://github.com/user-attachments/assets/48705f1e-4870-4c91-bdbd-14dd09433863" />


TODO:
- Server
  - [x] Read config
  - [x] Spawn individual task for each config kind
  - [x] Config file
  - [ ] Google News URL resolve
  - [ ] SQL database
  - [ ] Server side output filter
  - [ ] PWA Push API handling
- RSS Source
  - [ ] Google News Domain based (i.e. lazy API)
  - [x] Hacker News
  - [x] Youtube
  - [x] Reddit
  - [x] Generic RSS
- UI
  - [ ] Multi-window
  - [ ] Content filtering (Filter the entire window and stored)
  - [ ] Search feature
  - [ ] PWA ~Push API~ Background sync on offline + periodic sync
  - [ ] WebSocket indication?
  - [ ] ehttp error handling 
