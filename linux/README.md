To install as a systemd service:

1. Edit `bliss-mixer.service` and change:
    - Path to where `bliss-mixer` is run from 
    - User `bliss-mixer` will run as
    - Path to bliss.db to match where you will have this stored
2. Copy `bliss-mixer.service` to `/etc/systemd/system`
3. `sudo systemctl daemon-reload`
4. `sudo systemcrl enable bliss-mixer`
5. `sudo systemctl start bliss-mixer`
