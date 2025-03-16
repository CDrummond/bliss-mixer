# HTTP API

BlissMixer provides a simple HTTP API for the mixing tracks and getting list of similar tracks.

## Mix

This API is used to provide a list of tracks based upon a provied list of seed tracks. This is the main
API used by the Lyrion `Don't Stop The Music` mixer. API request payload is a JSON string, and the
response is a new-line separated list of tracks.

| Field       | Type                      | Description                                                 | Default         |
| ----------- | ------------------------- |-------------------------------------------------------------|-----------------|
| count       | Int                       | Number of tracks to return (1-50).                          | `5`             |
| filtergenre | Bool (1/0)                | Filter tracks on `genregroups`.                             | `0`             |
| filterxmas  | Bool (1/0)                | Exclude `Christmas` genre tracks, unless in december.       | `0`             |
| min         | Int                       | Min track duration (seconds).                               | _(none)_        |
| max         | Int                       | Max track duration (seconds).                               | _(none)_        |
| maxbmpdiff  | Int                       | Max BPM difference between seed track and similar tracks.   | _(none)_        |
| tracks      | Array of strings          | Seed tracks used for mix.                                   | _(mandatory)_   |
| previous    | Array of strings          | Current tracks in queue, used to prevent repeats, etc.      | `[]`            |
| shuffle     | Bool (1/0)                | Shuffle list of similar tracks.                             | `0`             |
| norepart    | Int                       | Don't repeat an artist for N tracks.                        | `0`             |
| norepalb    | Int                       | Don't repeat an album for N tracks.                         | `0`             |
| genregroups | Array of array of strings | List of genre groups, used when filering on genre.          | _(mandatory)_   |
| forest      | Bool (1/0)                | Use 'extended isolation forest', if more than 1 seed track. | `0`             |


Notes:
* If `shuffle` is enabled then the mixer will locate more than `count` similar tracks, shuffle the list, and take the first `count` tracks of the shuffled list.
* `norepart` and `norepalb` require `previous` list of tracks to be supplied.
* Set `maxbmpdiff` to 0 (or omit the field) to disable BPM difference checking.
* Set `min` or `max` to 0 (or omit the fields) to disable filtering on track duration.


Example request:
```json
{
    "count": 5,
    "filtergenre": 1,
    "filterxmas": 1,
    "min": 60,
    "max": 300,
    "maxbpmdiff": 0,
    "tracks": ["ArtistA/Album/Track1.ogg", "ArtistB/Album/Track1.ogg"],
    "previous": ["ArtistA/Album/Track2.ogg", "ArtistC/Album/Track2.ogg"],
    "shuffle": 1,
    "norepart": 10,
    "norepalb": 10,
    "genregroups": [
        [
            "Rock", "Metal"
        ],
        [
            "Dance", "R&B", "Pop"
        ]
    ],
    "forest": 1
}
```

Send via CURL:
```bash
curl 'http://localhost:12000/api/list' --compressed -X POST -H 'Content-Type: application/json' --data-raw '{"count":5,"filtergenre":1,"filterxmas":1,"min":60,"max":300,"maxbpmdiff":0,"tracks":["ArtistA/Album/Track1.ogg","ArtistB/Album/Track1.ogg"],"previous":["ArtistA/Album/Track2.ogg","ArtistC/Album/Track2.ogg"],"shuffle":1,"norepart":10,"norepalb":10,"genregroups":[["Rock","Metal"],["Dance","R&B","Pop"]],"forest":1}'
```


Example response:

```text
ArtistZ/AlbumY/Track5.ogg
ArtistW/AlbumG/Track9.ogg
ArtistD/AlbumA/Track2.ogg
ArtistP/AlbumH/Track10.ogg
ArtistF/AlbumE/Track2.ogg
```

## List

This API is used to query for an ordered list of tracks similar to provided track. API request payload
is a JSON string, and the response is a new-line separated list of tracks.

| Field       | Type                      | Description                                               | Default       |
| ----------- | ------------------------- |-----------------------------------------------------------|---------------|
| count       | Int                       | Number of tracks to return (1-50).                        | `5`           |
| filtergenre | Bool (1/0)                | Filter tracks on `genregroups`.                           | `0`           |
| min         | Int                       | Min track duration (seconds).                             | _(none)_      |
| max         | Int                       | Max track duration (seconds).                             | _(none)_      |
| maxbmpdiff  | Int                       | Max BPM difference between seed track and similar tracks. | _(none)_      |
| track       | String                    | Track to get similar tracks of.                           | _(mandatory)_ |
| genregroups | Array of array of strings | List of genre groups, used when filering on genre.        | _(mandatory)_ |
| byartist    | Bool (1/0)                | Restrict to tracks of same artist.                        | _(mandatory)_ |

Notes:
* Set `maxbmpdiff` to 0 (or omit the field) to disable BPM difference checking.
* Set `min` or `max` to 0 (or omit the fields) to disable filtering on track duration.


Example request:

Get 2 tracks (from 30 seconds to 5 minutes), from the same artist, similar to "Artist/Album/Track.ogg".

```json
{
    "count": 2,
    "filtergenre": 1,
    "min": 60,
    "max": 300,
    "maxbpmdiff": 0,
    "track":"Artist/Album/Track.ogg",
    "genregroups": [
        [
            "Rock", "Metal"
        ],
        [
            "Dance", "R&B", "Pop"
        ]
    ],
    "byartist": 1
}
```

Send via CURL:
```bash
curl 'http://localhost:12000/api/list' --compressed -X POST -H 'Content-Type: application/json' --data-raw '{"count":2,"filtergenre":1,"min":60,"max":300,"maxbpmdiff":0,"track":"Artist/Album/Track.ogg","genregroups":[["Rock","Metal"],["Dance","R&B","Pop"]],"byartist":0}'
```

Example response:

```text
ArtistZ/AlbumY/Track5.ogg
ArtistW/AlbumG/Track9.ogg
```
