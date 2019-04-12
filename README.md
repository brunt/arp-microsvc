# ARP Microservice
A simple web API to do an [arp](https://en.wikipedia.org/wiki/Address_Resolution_Protocol) scan on the local network and get human-readable results of vendor hardware on the local wifi network.

## What it does
* API sends ARP request to each IP address on local network and receives MAC addresses
* API matches each MAC address against cached list in app state
* API calls macvendors.com for each MAC that is found that is not in app state
* Results from macvendors are added to cache
* Response is returned to requester

GET /arp
```json
{
  "results": [
    {
      "mac_addr": "98:01:a7",
      "vendor_name": "Apple Inc."
    }
  ]
}
```