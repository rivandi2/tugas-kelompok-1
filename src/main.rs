use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;
use chrono::{offset::TimeZone, DateTime, Local, NaiveDateTime};

#[derive(Debug)]
struct Perabot {
    id: i64,
    name: String,
    stock: i64,
}

struct Perabots {
    inner: HashMap<i64, Perabot>,
}
impl Perabots {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    fn tambah(&mut self, id: i64, name: &str, stock_tambah: i64){
        let perabot = self.inner.remove(&id).unwrap();
        println!("Total stok {} kini: {}", name, &perabot.stock+&stock_tambah);     
        self.inner.insert(
            perabot.id,
            Perabot {
                id: perabot.id,
                name: perabot.name,
                stock: perabot.stock + stock_tambah,
            },
        );   
    }

    fn keluar(&mut self, id: i64, name: &str, stock_keluar: i64){
        let perabot = self.inner.remove(&id).unwrap();
        println!("Total stok {} kini: {}", name, &perabot.stock-&stock_keluar);     
        self.inner.insert(
            perabot.id,
            Perabot {
                id: perabot.id,
                name: perabot.name,
                stock: perabot.stock - stock_keluar,
            },
        );
    }

    fn edit(&mut self, id: i64, name: &str, stock: i64){
        self.inner.insert(
            id,
            Perabot {
                id,
                name: name.to_string(),
                stock,
            },
        );
    }

    fn add(&mut self, perabot: Perabot){
        self.inner.insert(perabot.id, perabot);
    }

    fn remove(&mut self, id: &i64) -> Option<Perabot> {
        self.inner.remove(&id)
    }

    fn into_vec(mut self) -> Vec<Perabot> {
        let mut perabots: Vec<_> = self.inner.drain().map(|e| e.1).collect();
        perabots.sort_by_key(|fd| fd.id);
        perabots
    }

    fn next_id(&self) -> i64 {
        let mut ids: Vec<_> = self.inner.keys().collect();
        ids.sort();
        match ids.pop() {
            Some(id) => id+1,
            None => 1,
        }
    }

    fn search(&mut self, name:&str) -> Option<i64> {
        let perabots: Vec<_> = self.inner.drain().map(|e| e.1).collect();
        for perabot in perabots {
            if perabot.name.to_lowercase() == name.to_lowercase(){
                return Some(perabot.id);
            }
        }
        None
    }

    fn search_stock(&mut self, name_buy: &str, stock_keluar: &i64) -> Result<i64, ParseError> {
        let perabots: Vec<_> = self.inner.drain().map(|e| e.1).collect();
        for perabot in perabots {
            if perabot.name == name_buy {
                if stock_keluar > &perabot.stock{
                    return Err(ParseError::StockPerabot);
                } else {
                    return Ok(perabot.id);
                }
            }    
        }
        Err(ParseError::FoundPerabot) 
    }

    fn search_empty(&mut self, name:&str) -> Result<i64, ParseError> {
        let perabots: Vec<_> = self.inner.drain().map(|e| e.1).collect();
        for perabot in perabots {
            if perabot.name.to_lowercase() == name.to_lowercase(){
                if perabot.stock == 0 {
                    return Ok(perabot.id);
                } else {
                    return Err(ParseError::StockPerabotAda);
                }
            }
        }
        Err(ParseError::FoundPerabot) 
    }
    
    fn sort_perabots(&mut self, id_del: i64) {
        let perabots: Vec<_> = self.inner.drain().map(|e| e.1).collect();
        for perabot in perabots {
            if perabot.id >= id_del + 1 {
                self.inner.insert(
                    perabot.id - 1,
                    Perabot {
                        id: perabot.id - 1,
                        name: perabot.name,
                        stock: perabot.stock,
                    },
                );
            } else{
                self.inner.insert(
                    perabot.id,
                    Perabot {
                        id: perabot.id,
                        name: perabot.name,
                        stock: perabot.stock,
                    },
                );
            }
        } 
    }      

}

#[derive(Error, Debug)]
enum ParseError {
    #[error("id must be a number: {0}")] InvalidId(#[from] std::num::ParseIntError),
    #[error("Perabot tidak ditemukan")] FoundPerabot,
    #[error("Stock perabot kurang atau habis")] StockPerabot,
    #[error("Stock belum kosong")] StockPerabotAda,
    #[error("Missing: {0}")] MissingField(String),
    #[error("Tidak ada perabotan dengan nama {0} pada list")] PerabotNotFound(String),
}

fn parse_perabot(perabot: &str) -> Result<Perabot, ParseError> {
    let fields: Vec<&str> = perabot.split(',').collect();
    let id = match fields.get(0) {
        Some(id) => i64::from_str_radix(id, 10)?,
        None => return Err(ParseError::FoundPerabot),
    };

    let name = match fields.get(1).filter(|name| **name != "") {
        Some(name) => name.to_string(),
        None => return Err(ParseError::MissingField("name".to_owned())),
    };

    let stock = match fields.get(2) {
        Some(stock) => i64::from_str_radix(stock, 10)?,
        None => return Err(ParseError::FoundPerabot),
    };

    Ok(Perabot {id,name,stock})
}

fn parse_perabots(perabots: String) -> Perabots {
    let mut pers = Perabots::new();
    for (_num, perabot) in perabots.split('\n').enumerate(){
        if perabot != ""{
            match parse_perabot(perabot){
                Ok(per) => pers.add(per),
                Err(_) =>{}
            }
        }
    }
    pers
}

fn load_perabots() -> std::io::Result<Perabots> {
    let mut file = File::open(PathBuf::from("src/bin/perabot.csv"))?;

    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    Ok(parse_perabots(buffer))
}

fn save_perabots(perabots: Perabots) -> std::io::Result<()>{
    let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(PathBuf::from("src/bin/perabot.csv"))?;

    file.write(b"id,name,stock\n")?;

    for perabot in perabots.into_vec().into_iter(){
        let line = format!("{},{},{}\n", perabot.id, perabot.name, perabot.stock);
        file.write(line.as_bytes())?;
    }

    file.flush()?;
    Ok(())
}

#[derive(Debug)]
struct History {
    id: i64,
    name: String,
    operation: String,
    stock: i64,
    time: DateTime<Local>,
}

struct Historys {
    inner: HashMap<i64, History>,
}

impl Historys {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    fn add(&mut self, history: History){
        self.inner.insert(history.id, history);
    }

    fn into_vec(mut self) -> Vec<History> {
        let mut historys: Vec<_> = self.inner.drain().map(|e| e.1).collect();
        historys.sort_by_key(|rp| rp.id);
        historys.reverse();
        historys
    }

    fn next_id(&self) -> i64 {
        let mut ids: Vec<_> = self.inner.keys().collect();
        ids.sort();
        match ids.pop() {
            Some(id) => id+1,
            None => 1,
        }
    }
}

fn parse_history(history: &str) -> Result<History, ParseError> {
    let fields: Vec<&str> = history.split(',').collect();
    let id = match fields.get(0) {
        Some(id) => i64::from_str_radix(id, 10)?,
        None => return Err(ParseError::FoundPerabot),
    };

    let name = match fields.get(1).filter(|name| **name != "") {
        Some(name) => name.to_string(),
        None => return Err(ParseError::MissingField("name".to_owned())),
    };

    let operation = match fields.get(2).filter(|operation| **operation != "") {
        Some(operation) => operation.to_string(),
        None => return Err(ParseError::MissingField("operation".to_owned())),
    };

    let stock = match fields.get(3) {
        Some(stock) => i64::from_str_radix(stock, 10)?,
        None => return Err(ParseError::StockPerabot),
    };

    let date_time = match fields.get(4) {
        Some(date_time) => NaiveDateTime::parse_from_str(date_time, "%Y-%m-%d %H:%M:%S %z").unwrap(),
        None => return Err(ParseError::StockPerabot),
    };

    let time: DateTime<Local> = Local.from_local_datetime(&date_time).unwrap();
    Ok(History {id,name,operation,stock,time})
}

fn parse_historys(historys: String) -> Historys {
    let mut hiss = Historys::new();
    for (_num, history) in historys.split('\n').enumerate(){
        if history != ""{
            match parse_history(history){
                Ok(his) => hiss.add(his),
                Err(_) =>{}
            }
        }
    }
    hiss
}

fn load_historys() -> std::io::Result<Historys> {
    let mut file = File::open(PathBuf::from("src/bin/history.csv"))?;

    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    Ok(parse_historys(buffer))
}

fn save_historys(historys: Historys) -> std::io::Result<()>{
    let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(PathBuf::from("src/bin/history.csv"))?;

    file.write(b"id,name,operation,stock,time\n")?;

    for history in historys.into_vec().into_iter(){
        let line = format!("{},{},{},{},{}\n", history.id, history.name, history.operation, history.stock, history.time);
        file.write(line.as_bytes())?;
    }

    file.flush()?;
    Ok(())
}

fn time_now() -> DateTime<Local> {
    let temptime = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let temp_time = NaiveDateTime::parse_from_str(&temptime, "%Y-%m-%d %H:%M:%S").unwrap();
    let datime: DateTime<Local> = Local.from_local_datetime(&temp_time).unwrap();
    datime
}

fn print_history(historys: Vec<History>, opsi: String){
    for history in historys{
        if history.operation == opsi {
            println!("{},{},{},{},{}", history.id, history.name, history.operation, history.stock, history.time);
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Mini WareHouse")]
struct Opt{
    #[structopt(subcommand)] cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    Masuk {
        name: String,
        stock: i64,
    },
    Keluar {
        name: String,
        stock: i64,
    },
    Hapus {
        name: String,
    },
    Edit{
        name: String,
        stock: i64,
    },
    List {},
    History{
        opsi: Option<String>,
    },
}

fn run(opt: Opt) -> Result<(), std::io::Error> {
    match opt.cmd {
        Command::Masuk {name, stock} => { 
            let mut pers = load_perabots()?;
            let mut hiss = load_historys()?;
            let results = pers.search(&name);
            let copy = name.clone();
            match results {
                Some(p) => {
                    let mut pers = load_perabots()?;
                    println!("Berhasil menambah perabotan {} sebanyak {} stok", &name, &stock);
                    pers.tambah(p, &name, stock);
                    save_perabots(pers)?;
                },
                None => {
                    let mut pers = load_perabots()?;
                    let next_id = pers.next_id();
                    println!("Berhasil menambahkan perabotan baru, {} dengan jumlah stok {}", &name, &stock);
                    pers.add(Perabot {
                        id: next_id,
                        name,
                        stock,
                    });
                    save_perabots(pers)?;
                },
            }
            let next_id = hiss.next_id();
            let datime = time_now();

            hiss.add(History {
                id: next_id,
                name: copy,
                operation: "masuk".to_string(),
                stock,
                time: datime,
            });
            save_historys(hiss)?;
        }

        Command::List { .. } => {
            let pers = load_perabots()?;
            let tmp = pers.into_vec();

            if tmp.len()==0 {
                println!("List perabotan kosong!");
            }  else {
                let pers = load_perabots()?;
                println!{"id,name,stock"};
                for perabot in pers.into_vec(){
                    println!("{},{},{}", perabot.id, perabot.name, perabot.stock);
                }
            }
        }

        Command::Edit {name, stock} => {
            let mut pers = load_perabots()?;
            let results = pers.search(&name);
            match results{
                Some(p) => {
                    let mut pers = load_perabots()?;
                    let mut hiss = load_historys()?;
                    println!("Berhasil mengupdate stok perabot {} menjadi {}", &name, &stock);
                    pers.edit(p, &name, stock);
                    save_perabots(pers)?;

                    let next_id = hiss.next_id();
                    let datime = time_now();
        
                    hiss.add(History {
                        id: next_id,
                        name,
                        operation: "edit".to_string(),
                        stock,
                        time: datime,
                    });
                    save_historys(hiss)?;
                }
                None => {
                    println!("{}", ParseError::PerabotNotFound(name));
                }
            }
        }

        Command::Keluar {name, stock} => { 
            let mut pers = load_perabots()?;
            let results = pers.search_stock(&name, &stock);
            match results {
                Ok(p) => {
                    let mut pers = load_perabots()?;
                    let mut hiss = load_historys()?;
                    println!("Perabotan {} berhasil dikeluarkan sebanyak {} stok", &name, &stock);
                    pers.keluar(p, &name, stock);
                    save_perabots(pers)?;

                    let next_id = hiss.next_id();
                    let datime = time_now();
        
                    hiss.add(History {
                        id: next_id,
                        name,
                        operation: "keluar".to_string(),
                        stock,
                        time: datime,
                    });
                    save_historys(hiss)?;
                },
                Err(e) => {
                    println!("{}", e);
                },
            }
        }

        Command::Hapus {name} => { 
            let mut pers = load_perabots()?;
            let results = pers.search_empty(&name);
            match results {
                Ok(p) => {
                    let mut pers = load_perabots()?;
                    let mut hiss = load_historys()?;
                    pers.remove(&p);
                    pers.sort_perabots(p);
                    save_perabots(pers)?;
                    println!("Perabotan {} berhasil dihapus dari daftar", &name);

                    let next_id = hiss.next_id();
                    let datime = time_now();
        
                    hiss.add(History {
                        id: next_id,
                        name,
                        operation: "hapus".to_string(),
                        stock: 0,
                        time: datime,
                    });
                    save_historys(hiss)?;
                },
                Err(e) => {
                    println!("{}", e);
                },
            }
        }

        Command::History { opsi } => {
            let hiss = load_historys()?;
            let tmp = hiss.into_vec();
            if tmp.len()==0{
                println!("History is empty");
            } else {
                println!{"id,name,operation,stock,time"};
                match opsi {
                    Some(p) => {
                        let hiss = load_historys()?;
                        let historys = hiss.into_vec();
                        if p.to_lowercase() == "masuk" || p.to_lowercase() == "keluar" ||  
                            p.to_lowercase() == "edit" || p.to_lowercase() == "hapus"
                        {
                            print_history(historys, p.to_lowercase());
                        } else {
                            println!("Invalid command!");
                        }
                    },
                    None => {
                        let hiss = load_historys()?;
                        for history in hiss.into_vec(){
                            println!("{},{},{},{},{}", history.id, history.name, history.operation, history.stock, history.time);
                        }
                    },
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        println!("an error occured: {}", e);
    } 
}