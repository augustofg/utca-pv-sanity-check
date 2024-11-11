use epics_ca::Context;
use epics_ca::ValueChannel;
use epics_ca::Error;
use epics_ca::types::{Value, EpicsEnum};
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

    let fofb_dcc_frame_err_pv_array: [_; 4*60] = core::array::from_fn(|i| {
        if i < 20*4 {
            ctx.connect::<i32>(format!("IA-{:02}RaBPM:BS-FOFBCtrl:DCCFMCFrameErrCntCH{}-Mon", i / 4 + 1, i % 4))
        } else {
            ctx.connect::<i32>(format!("IA-{:02}RaBPM:BS-FOFBCtrl:DCCP2PFrameErrCntCH{}-Mon", (i - 20*4) / 8 + 1, i % 8))
        }
    });

    let fofb_dcc_frame_err_ch_array = join_all(fofb_dcc_frame_err_pv_array).await;
    println!("All DCCFMCFrameErrCntCH and DCCP2PFrameErrCntCH PVs joined!");

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

    let afc_timing_ch_arr: [u8; 4] = [0, 1, 2, 5];

    let afc_timing_fofb_trig_width_pv_array: [_; 4*20] = core::array::from_fn(|i| {
        ctx.connect::<f64>(format!("IA-{:02}RaBPM:TI-AMCFPGAEVR:AMC{}Width-RB", i / 4 + 1, afc_timing_ch_arr[i % 4]))
    });

    let afc_timing_fofb_trig_width_pv_array = join_all(afc_timing_fofb_trig_width_pv_array).await;
    println!("All TI-AMCFPGAEVR:AMCxWidth-RB PVs joined!");

    err_cnt = 0;
    for ch in afc_timing_fofb_trig_width_pv_array {
        let mut ch_con = ch.unwrap();
        let value = ch_con.get().await.unwrap();
        if value < 0.047 {
            err_cnt = err_cnt + 1;
            println!("{}: {}", ch_con.name().to_str().unwrap(), value);
        }
    }

    if err_cnt == 0 {
        println!("All TI-AMCFPGAEVR:AMCxWidth-RB are ok!");
    }

    let afc_timing_fofb_trig_en_pv_array: [_; 4*20] = core::array::from_fn(|i| {
        ctx.connect::<EpicsEnum>(format!("IA-{:02}RaBPM:TI-AMCFPGAEVR:AMC{}State-Sts", i / 4 + 1, afc_timing_ch_arr[i % 4]))
    });

    let afc_timing_fofb_trig_en_pv_array = join_all(afc_timing_fofb_trig_en_pv_array).await;
    println!("All TI-AMCFPGAEVR:AMCxState-Sts PVs joined!");

    err_cnt = 0;
    for ch in afc_timing_fofb_trig_en_pv_array {
        let mut ch_con = ch.unwrap();
        let value = ch_con.get().await.unwrap();
        if value.0 != 1 {
            err_cnt = err_cnt + 1;
            println!("{}: {}", ch_con.name().to_str().unwrap(), value.0);
        }
    }

    if err_cnt == 0 {
        println!("All TI-AMCFPGAEVR:AMCxState-Sts are ok!");
    }
}
