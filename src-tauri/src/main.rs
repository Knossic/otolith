// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use watcher::hello_from_watcher;

fn main() {
    println!("{}", hello_from_watcher());
    otolith_lib::run()
}
