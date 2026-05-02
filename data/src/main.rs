use data::*;

fn main() -> Result<(), CanError> {
    // let msg = MotorCmd::new(3, 3)?;
    // let mux = SensorSonarsMsgMux0::new(0.0, 1.0, 2.0, 2.5)?;
    // let mut msg = SensorSonarsMsg::new(
    //     54u16,
    //     SensorSonarsMsgMux::V0(mux),
    // )?;

    // println!("{}", msg.sensor_sonars_err_count());
    // let mux = msg.mux()?;

    // match &mux {
    //     SensorSonarsMsgMux::V0(v) => {
    //         println!("{}", v.sensor_sonars_left());
    //         println!("{}", v.sensor_sonars_middle());
    //         println!("{}", v.sensor_sonars_right());
    //         println!("{}", v.sensor_sonars_rear());
    //     },
    //     _ => (),
    // }

    // let mux = SensorSonarsMsgMux1::new(2.5, 2.0, 1.0, 1.0)?;
    // msg.set_mux1(mux)?;

    // println!();
    // println!("{}", msg.sensor_sonars_err_count());

    // match &msg.mux()? {
    //     SensorSonarsMsgMux::V1(v) => {
    //         println!("{}", v.sensor_sonars_no_filt_left());
    //         println!("{}", v.sensor_sonars_no_filt_middle());
    //         println!("{}", v.sensor_sonars_no_filt_right());
    //         println!("{}", v.sensor_sonars_no_filt_rear());
    //     },
    //     _ => (),
    // }

    // println!("{}", msg.motor_cmd_steer());

    // let mut msg = SensorSonars::new(0, 54)?;
    // let mut mux = SensorSonarsSensorSonarsMuxM0::new();
    // mux.set_sensor_sonars_left(0.0)?;
    // mux.set_sensor_sonars_middle(1.0)?;
    // mux.set_sensor_sonars_right(2.0)?;
    // mux.set_sensor_sonars_rear(2.5)?;
    // msg.set_m0(mux)?;

    // match msg.sensor_sonars_mux()? {
    //     SensorSonarsSensorSonarsMuxIndex::M0(v) => {
    //         println!("{}", v.sensor_sonars_left());
    //         println!("{}", v.sensor_sonars_middle());
    //         println!("{}", v.sensor_sonars_right());
    //         println!("{}", v.sensor_sonars_rear());
    //     },
    //     _ =>(),
    // }

    // println!();

    // let mut mux = SensorSonarsSensorSonarsMuxM1::new();
    // mux.set_sensor_sonars_no_filt_left(2.5)?;
    // mux.set_sensor_sonars_no_filt_middle(2.0)?;
    // mux.set_sensor_sonars_no_filt_rear(1.0)?;
    // mux.set_sensor_sonars_no_filt_right(1.0)?;
    // msg.set_m1(mux)?;

    // match msg.sensor_sonars_mux()? {
    //     SensorSonarsSensorSonarsMuxIndex::M1(v) => {
    //         println!("{}", v.sensor_sonars_no_filt_left());
    //         println!("{}", v.sensor_sonars_no_filt_middle());
    //         println!("{}", v.sensor_sonars_no_filt_right());
    //         println!("{}", v.sensor_sonars_no_filt_rear());
    //     },
    //     _ =>(),
    // }

    Ok(())
}
