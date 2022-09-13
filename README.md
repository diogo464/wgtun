# wgtun

## server
+ accepts tcp connection
+ received rle messages
+ sends message payload as a udp packet to localhost:51820
+ received udp packets from that socket and sends them as rle to the client
wgtun server
	--target <sock addr> 	# SockAddr to send the udp packets to, defaults to localhost:51820

## client
+ attempts tcp connection
+ close the connection after x seconds of inactivity
+ binds udp socket and listens for incoming datagrams
+ sends datagrams as rle message to the tcp stream, if the tcp stream is not connected drop them
+ receives rle messages and sends them to the socket address that sent the first udp packet to the bound socket
wgtun client
	--port <number> 		# udp port to bind to on the localhost
	--timeout <seconds> 	# Number of seconds of inactivity before closing the tcp connection
	--server <sock addr> 	# SockAddr of the server to connect to, port defaults to 51820
