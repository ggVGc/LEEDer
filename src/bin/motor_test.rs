use std::thread;
use std::time::Duration;

use pyo3::prelude::*;
use pyo3::types::PyTuple;

fn main() -> PyResult<()> {
    // let arg1 = "arg1";
    // let arg2 = "arg2";
    // let arg3 = "arg3";
    pyo3::prepare_freethreaded_python();

    let code = std::fs::read_to_string("src/motors.py")?;

    Python::with_gil(|py| {
        let motors = PyModule::from_code(py, &code, "motors.py", "")?;

        // let example: Py<PyAny> = motors.getattr("example")?.into();
        let test: Py<PyAny> = motors.getattr("test")?.into();

        loop {
            // example.call0(py)?;
            match test.call0(py) {
                Ok(_) => (),
                Err(err) => {
                    println!("Error: {:?}", err);
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
        // call object without any arguments

        // call object with PyTuple
        // let args = PyTuple::new(py, &[arg1, arg2, arg3]);
        // fun.call1(py, args)?;

        // // pass arguments as rust tuple
        // let args = (arg1, arg2, arg3);
        // fun.call1(py, args)?;
        Ok(())
    })
}
