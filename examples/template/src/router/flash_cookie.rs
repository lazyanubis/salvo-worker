use flash::FlashDepotExt;
use salvo_worker::salvo::*;
use std::fmt::Write;

#[handler]
pub async fn set_flash(depot: &mut Depot, res: &mut Response) {
    let flash = depot.outgoing_flash_mut();
    flash.info("Hey there!").debug("How is it going?");
    res.render(Redirect::other("/flash_cookie/get"));
}

#[handler]
pub async fn get_flash(depot: &mut Depot, _res: &mut Response) -> String {
    let mut body = String::new();
    if let Some(flash) = depot.incoming_flash() {
        for message in flash.iter() {
            writeln!(body, "{} - {}", message.value, message.level).unwrap();
        }
    }
    body
}
