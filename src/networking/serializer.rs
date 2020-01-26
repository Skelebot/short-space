extern crate flatbuffers;
use nalgebra as na;
use super::message::ReliableMessage;

use super::client_packet_generated as cpg;

///Create a ready-to-send byte packet from client input
pub fn create_client_packet(
    reliable_messages: Option<Vec<ReliableMessage>>,
) -> Vec<u8>{

//    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);

//    //---INPUT BUFFER---
//    let keeb_buf_pack = builder.create_vector(&input_message.keeb_buf);
//    let mouse_delta_pack = &cpg::Vec3::new(
//        input_message.dir_delta.0,
//        input_message.dir_delta.1,
//        input_message.dir_delta.2,
//    );
//    let pack_input = 
//        cpg::InputBuffer::create(
//            &mut builder,
//            &cpg::InputBufferArgs {
//                keeb_buf: Some(keeb_buf_pack),
//                mouse_delta: Some(mouse_delta_pack),
//            }
//        );
//
//    //---RELIABLE MESSAGES---
//    let mut pack_rel_messages = Vec::new();
//    if reliable_messages.is_some() {
//        for rel_msg in reliable_messages.unwrap() {
//            let data_vec = builder.create_vector(&rel_msg.data);
//            let pack_msg = cpg::ReliableMessage::create(
//                &mut builder,
//                &cpg::ReliableMessageArgs {
//                    id: rel_msg.id,
//                    type_: rel_msg.type_,
//                    data: Some(data_vec)
//                }
//            );
//            pack_rel_messages.push(pack_msg);
//        }
//    }
//
//    //---CLIENT PACKET---
//    let reliable_vec = builder.create_vector(&pack_rel_messages[..]);
//    let client_packet = cpg::ClientPacket::create(
//        &mut builder,
//        &cpg::ClientPacketArgs {
//            reliable: Some(reliable_vec),
//            unreliable: Some(pack_input),
//        }
//    );
//
//    builder.finish(client_packet, None);
//    let data = builder.finished_data();
//    let mut packet: Vec<u8> = Vec::new();
//    for b in data {
//        packet.push(*b);
//    }
//    return packet;
        Vec::new()
}
