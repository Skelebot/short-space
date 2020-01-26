//use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
//pub mod serializer;
//pub mod message;
//#[allow(dead_code, unused_imports)]
//mod client_packet_generated;
//use self::message as msg;
//use std::thread;
//use std::sync::mpsc;
//use std::time::Duration;
//
//pub const CMD_RATE: u32 = 30; //Command send rate, times/second
//
//#[derive(Debug, Fail)]
//pub enum NetError {
//    #[fail(display = "Failed to bind UDP socket on port {}", port)]
//    SocketBindingFailed {
//        port: String
//    },
//    #[fail(display = "Socket connection failed: {}", addr)]
//    SocketConnectionFailed {
//        addr: String
//    },
//    #[fail(display = "Tried sending data on unconnected socket")]
//    SocketNotConnected,
//    #[fail(display = "Recv function failed: {:?}", error)]
//    RecvFunctionFailed {
//        #[cause]
//        error: std::io::Error,
//    }
//}
//
//pub struct ServerConnection {
//    address: SocketAddr,
//    socket: Option<UdpSocket>,
//}
//
//impl ServerConnection {
//    pub fn new(addr: &str) -> Self {
//        ServerConnection {
//            address: addr.to_socket_addrs().unwrap().next().unwrap(),
//            socket: None,
//        }
//    }
//
//    pub fn connect(&mut self, send_port: &str) -> Result<(), NetError> {
//        let socket = UdpSocket::bind("localhost:".to_owned()+send_port)
//            .map_err(|_| NetError::SocketBindingFailed{port: String::from(send_port)})?;
//        socket.connect(self.address)
//            .map_err(|_| NetError::SocketConnectionFailed{addr: self.address.to_string()})?;
//        self.socket = Some(socket);
//        Ok(())
//    }
//
//    pub fn start_listening(&mut self) -> Result<mpsc::Receiver<msg::ServerPacket>, NetError> {
//        if self.socket.is_none() { return Err(NetError::SocketNotConnected); }
//
//        let (tx, rx) = mpsc::channel();
//        //let socket = self.socket.as_ref().unwrap();
//        let address = self.address.clone();
//
//        //thread::JoinHandle
//        let handle = thread::spawn(move || {
//            //TODO: Timeout? Sleep between loops?
//            loop {
//                let mut buffer: [u8; 1024] = [0; 1024];
//                //let num_bytes = socket.recv(&mut buffer);
//                //if num_bytes.is_ok() && num_bytes.unwrap() > 0 {
//                    //let packet = decode_server_packet(&buffer);
//                    //tx.send(packet).unwrap();
//                //}
//            }
//        });
//        return Ok(rx);
//    }
//
//    pub fn send_data(&self, data: &[u8]) -> Result<(), NetError> {
//        if self.socket.is_none() { return Err(NetError::SocketNotConnected); }
//        &self.socket.as_ref().unwrap()
//            .send(data).map_err(|_| NetError::SocketNotConnected)?;
//        Ok(())
//    }
//
//    pub fn recv_data(&self, buffer: &mut [u8]) -> Result<usize, NetError> {
//        if self.socket.is_none() { return Err(NetError::SocketNotConnected); }
//        let num_bytes = self.socket.as_ref().unwrap()
//            .recv(buffer).map_err(|e| NetError::RecvFunctionFailed{error: e})?;
//        println!("Recieved data: {:?}", buffer);
//        Ok(num_bytes)
//    }
//}
