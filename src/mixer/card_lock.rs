use dbus::Error as DBusError;
use dbus::blocking::Connection;

use std::sync::Mutex;
use std::mem;

use super::reservedevice1::*;


pub struct CardLock {

}

struct CardLockServer {
    id: CardId,
    lock: Mutex<Option<()>>,
    priority: i32,
    device_name: String,
}

impl OrgFreedesktopReserveDevice1 for CardLockServer {
    fn request_release(&self, priority: i32) -> Result<bool, dbus_tree::MethodErr> {
        if priority > self.priority {
            //Ditch Card here
            mem::swap(&mut None, &mut self.lock.lock().unwrap());
            return Ok(true);
        } else {
            return Ok(false);
        }
    }
    fn priority(&self) -> Result<i32, dbus_tree::MethodErr> {
        Ok(self.id)
    }
    fn application_name(&self) -> Result<String, dbus_tree::MethodErr> {
        Ok("JackCTL".to_owned())
    }
    fn application_device_name(&self) -> Result<String, dbus_tree::MethodErr> {
        Ok(self.device_name)
    } 
}

use super::CardId;

impl CardLock {
    pub fn aquire(card_id: CardId) -> Result<Self, () > {
        Err(())
    }

    pub fn create_server() -> Result<(), DBusError> {
        let factory = dbus_tree::Factory::new_fn::<()>();
        let interface = factory.interface("org.freedeskop.reservedevice1.audio0", ());

        let lock_server = CardLockServer::default();

        org_freedesktop_reserve_device1_server(&factory, (), |x| { dbg!(x); todo!(); &() } );
        
        let c = Connection::new_session()?;
        c.request_name("", false, true, false)?;
        // let mut cr = Crossroads::new();
        // let token = cr.register("com.example.dbustest", |b| {
        //     b.method("Hello", ("name",), ("reply",), |_, _, (name,): (String,)| {
        //         Ok(format!("Hello {}!", name))
        //     });
        // });
        // cr.insert("/hello", &[token], ());
        // cr.serve(&c)?;
        Ok(())
    }
}