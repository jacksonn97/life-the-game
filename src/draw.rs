#[allow(unused)]

use std::{
    time::Duration,
    thread,
    sync::{
        Arc,
        Mutex,
        atomic::{ AtomicBool, AtomicU64, Ordering },
        mpsc,
    },
    collections::LinkedList,
};

use crossterm::{
    terminal::{ self, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode, SetTitle, },
    cursor::{ SavePosition, RestorePosition, Show, Hide },
    execute,
    event::{
        self,
        Event,
        KeyModifiers,
        KeyCode, KeyEventKind,

    },
};

use crate::proc::Field;

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct Rect{
    w: u16, // j
    h: u16, // i
}

pub struct App {
    pub field: Mutex<Field>,
    pub should_exit: AtomicBool,
    pub pause: AtomicBool,
    maxgen: AtomicU64,
    pub upd_timeout: AtomicU64,
}

impl Rect {
    #[inline]
    pub fn new(width: u16, height: u16) -> Self {
        Rect{w: width, h: height}
    }

    #[inline]
    pub fn term_size() -> Self {

        let (width, height) = terminal::size().unwrap();
        Rect{w: width, h: height}
    }

    #[inline]
    pub fn w(&self) -> u16 {
        self.w
    }

    #[inline]
    pub fn h(&self) -> u16 {
        self.h
    }

    #[inline]
    pub fn unwrap(&self) -> (u16, u16) {
        (self.w, self.h)
    }
}

impl App {

    #[inline]
    pub fn new(field: Field, maxgen: u64) -> Self {
        App {
            field: Mutex::new(field),
            should_exit: false.into(),
            pause: false.into(),
            maxgen: maxgen.into(),
            upd_timeout: 450.into(),
        }
    }

    #[inline]
    pub fn maxgen(&self) -> u64 {
        self.maxgen.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn should_exit(&self) -> bool {
        self.should_exit.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn pause(&self) -> bool {
        self.pause.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn upd_timeout(&self) -> u64 {
        self.upd_timeout.load(Ordering::Relaxed)
    }

}

pub fn run(a: App) -> Result<()> {

    runup()?;
    clear()?;
    let d = draw(a);
    shutdown()?;
    d?;
    Ok(())
}

fn runup() -> Result<()> {
    execute!(std::io::stderr(), EnterAlternateScreen, SetTitle("Life a game"), Hide)?;
    enable_raw_mode()?;
    clear()?;
    execute!(std::io::stdout(), SavePosition)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;
    Ok(())
}

fn draw(a: App) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let a = Arc::new(a);

    let arc_ticks = Arc::clone(&a);
    let arc_keys = Arc::clone(&a);

    let _ = thread::Builder::new().name("Tick machine".into()).spawn(move || {
        let mut field = arc_ticks.field.lock().unwrap();
        let maxgen = arc_ticks.maxgen();
        for _ in 0..maxgen {
            tx.send(field.clone()).unwrap();
            field.tick();
        }
    });


    let _ = thread::Builder::new().name("Keyboard input".into()).spawn(move || {
        let a = arc_keys;
        static DELAYS: [u64; 14] = [1, 10, 20, 40, 60, 100, 150, 200, 300, 450, 800, 1200, 1500, 2000];
        let mut current_delay = LinkedList::from(DELAYS);
        for _ in 0..7 {
            let _ = current_delay.front();
        }
        loop {
            let _ = hotkeys(&a, &mut current_delay);
        }

    });

    let mut gen = 0u64;
    while gen < a.maxgen() {

        if a.should_exit() {
            break
        }

        if !a.pause() {
            gen += 1;
            let field = rx.recv().unwrap();

            sleep_ms(a.upd_timeout());
            clear()?;
            for c in field.data() {
                for r in c {
                    if *r {
                        print!("#");
                    } else {
                        print!(" ");
                    }
                }
                print!("\n\r");
            }
        }

    }
    Ok(())
}

fn clear() -> Result<()> {
    use terminal::{ Clear, ClearType };
    use std::io::stdout;

    execute!(stdout(), Clear(ClearType::Purge))?;
    execute!(stdout(), RestorePosition)?;
    Ok(())
}

fn sleep_ms(t: u64) {
    thread::sleep(Duration::from_millis(t))
}


fn hotkeys(a: &Arc<App>, del: &mut LinkedList<u64>) -> Result<()> {
    if event::poll(Duration::from_millis(150))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('c') => a.should_exit.store(true, Ordering::Relaxed),
                        _ => {},
                    }
                } else {
                    match key.code {
                        KeyCode::Char('p') => {
                            let p = a.pause();
                            a.pause.store(!p, Ordering::Relaxed);
                            if p {
                                println!("Paused!\r");
                            }
                        },
                        KeyCode::Char('j') => a.upd_timeout.store(*del.front().unwrap(), Ordering::Relaxed),
                        KeyCode::Char('k') => a.upd_timeout.store(*del.back().unwrap(), Ordering::Relaxed),
                        _ => {},
                    }
                }
            }
        }
    }
    Ok(())
}
