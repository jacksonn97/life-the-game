
use std::{
    time::Duration,
    thread,
    sync::{
        Arc,
        mpsc,
    },
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

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;

// Incapsulate for trust
pub use rect::Rect;
mod rect {

    #[derive(PartialEq, PartialOrd, Clone, Copy)]
    pub struct Rect{
        w: u16, // j
        h: u16, // i
    }

    impl Rect {
        #[inline]
        pub fn new(width: u16, height: u16) -> Self {
            Rect{w: width, h: height}
        }

        #[inline]
        pub fn term_size() -> Self {

            let (width, height) = crossterm::terminal::size().unwrap();
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

}

pub use app::App;
mod app {

    use crate::proc::Field;
    use std::sync::{
        Mutex,
        atomic::{
            AtomicU64,
            AtomicU16,
            AtomicBool,
            Ordering,
        }
    };

    pub struct App {
        field: Mutex<Field>,
        should_exit: AtomicBool,
        pause: AtomicBool,
        maxgen: AtomicU64,
        upd_timeout: AtomicU64,
        frames: AtomicU16,
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
                frames: 6.into(),
            }
        }

        #[inline]
        pub fn maxgen(&self) -> u64 {
            self.maxgen.load(Ordering::Relaxed)
        }

        #[inline]
        pub fn is_should_exit(&self) -> bool {
            self.should_exit.load(Ordering::Relaxed)
        }

        #[inline]
        pub fn is_paused(&self) -> bool {
            self.pause.load(Ordering::Relaxed)
        }

        #[inline]
        pub fn pause(&self) {
            self.pause.store(self.pause.load(Ordering::SeqCst), Ordering::SeqCst)
        }

        #[inline]
        pub fn upd_timeout(&self) -> u64 {
            self.upd_timeout.load(Ordering::Relaxed)
        }

        #[inline]
        pub fn need_frame(&self) {
            self.frames.fetch_sub(1, Ordering::SeqCst);
        }

        #[inline]
        pub fn add_frame(&self) {
            self.frames.fetch_add(1, Ordering::SeqCst);
        }

        #[inline]
        pub fn frames(&self) -> u16 {
            self.frames.load(Ordering::SeqCst)
        }

        #[inline]
        pub fn should_exit(&self) {
            self.should_exit.store(true, Ordering::Relaxed)
        }
        
        #[inline]
        pub fn store_timeout(&self, t: u64) {
            self.upd_timeout.store(t, Ordering::Relaxed)
        }

        #[inline]
        pub fn field(&self) -> &Mutex<Field> {
            &self.field
        }
    }

}

pub struct TimeoutIter {
    index: usize,
    vec: Vec<u64>,
    l: usize,
}

impl TimeoutIter {

    #[inline]
    pub fn new(vec: Vec<u64>, start_pos: usize) -> Self {
        Self {
            index: start_pos,
            l: vec.len() - 1,
            vec,
        }
    }

    #[inline]
    pub fn next(&mut self) -> u64 {
        let mut u = self.index;
        if self.index == self.l {
            self.index = 0;
            u = 0;
        } else {
            self.index += 1;
            u += 1;
        }
        self.vec[u]
    }

    #[inline]
    pub fn prev(&mut self) -> u64 {
        let mut u = self.index;
        if self.index == 0 {
            self.index = self.l;
            u = self.l;
        } else {
            self.index -= 1;
            u -= 1;
        }
        self.vec[u]
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
        let mut field = arc_ticks.field().lock().unwrap();
        let maxgen = arc_ticks.maxgen();

        for _ in 0..maxgen {
            if 48 > arc_ticks.frames() {
                tx.send(field.clone()).unwrap();
                field.tick();
                arc_ticks.add_frame();
            } else {
                thread::sleep(Duration::from_millis(50));
            }
        }
    });


    let _ = thread::Builder::new().name("Keyboard input".into()).spawn(move || {
        let a = arc_keys;
        let d = [0, 1, 2, 4, 8, 10, 15, 20, 25, 30, 40, 50, 80, 100, 1000];
        let mut current_delay = TimeoutIter::new(d.into(), 9);
        loop {
            let _ = hotkeys(&a, &mut current_delay);
        }

    });

    let mut gen = 0u64;
    let mut f = !a.is_paused();
    while gen < a.maxgen() {

        if a.is_should_exit() {
            break
        }
        if !a.is_paused() {
            gen += 1;
            a.need_frame();
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
            f = true;
        } else {
            if f {
                f = false;
                println!("Paused!\r")
            }
            thread::sleep(Duration::from_millis(500));
        }

        if a.is_should_exit() {
            break
        }

    }
    Ok(())
}

#[inline]
fn clear() -> Result<()> {
    use terminal::{ Clear, ClearType };
    use std::io::stdout;

    execute!(stdout(), Clear(ClearType::Purge))?;
    execute!(stdout(), RestorePosition)?;
    Ok(())
}

#[inline]
fn sleep_ms(t: u64) {
    thread::sleep(Duration::from_millis(t))
}


fn hotkeys(a: &Arc<App>, del: &mut TimeoutIter) -> Result<()> {
    if event::poll(Duration::from_millis(150))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('c') => a.should_exit(),
                        _ => {},
                    }
                } else {
                    match key.code {
                        KeyCode::Char('p') => {
                            a.pause();
                        },
                        KeyCode::Char('j') => a.store_timeout(del.prev()),
                        KeyCode::Char('k') => a.store_timeout(del.next()),
                        KeyCode::Char('q') => a.should_exit(),
                        _ => {},
                    }
                }
            }
        }
    }
    Ok(())
}
