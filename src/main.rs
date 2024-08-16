use epics_ca::Context;
use epics_ca::ValueChannel;
use epics_ca::Error;
use epics_ca::types::Value;
use futures::future::join_all;
use std::format;
use std::ffi::CString;

struct EpicsContextBundle {
    ctx: Context,
}

impl EpicsContextBundle {
    pub fn new() -> Result<Self, Error> {
        let ctx = Context::new()?;
        Ok(Self {
            ctx
        })
    }

    pub async fn connect<V: Value + ?Sized>(&self, pv_name: String) -> Result<ValueChannel<V>, Error> {
        self.ctx.connect::<V>(CString::new(pv_name).unwrap().as_c_str()).await
    }
}

#[async_std::main]
async fn main() {
    let ctx = EpicsContextBundle::new().unwrap();

    let fofb_dcc_frame_err_pv_array: [_; 4*20] = core::array::from_fn(|i| {
        ctx.connect::<i32>(format!("IA-{:02}RaBPM:BS-FOFBCtrl:DCCFMCFrameErrCntCH{}-Mon", i / 4 + 1, i % 4))
    });

    let fofb_dcc_frame_err_ch_array = join_all(fofb_dcc_frame_err_pv_array).await;
    println!("All DCCFMCFrameErrCntCH PVs joined!");

    let mut err_cnt = 0;
    for ch in fofb_dcc_frame_err_ch_array {
        let mut ch_con = ch.unwrap();
        let value = ch_con.get().await.unwrap();
        if value != 0 {
            err_cnt = err_cnt + 1;
            println!("{}: {}", ch_con.name().to_str().unwrap(), value);
        }
    }

    if err_cnt == 0 {
        println!("All DCCFMCFrameErrCnt are ok!");
    }
}
