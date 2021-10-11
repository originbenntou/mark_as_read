use crossterm::{
    event::{
        self,
        Event as CEvent,
        KeyEvent
    }
};
use std::{
    sync::mpsc,
    thread,
    time::{
        Duration,
        Instant
    }
};

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<KeyEvent>>,
    _tx: mpsc::Sender<Event<KeyEvent>>,
}

impl Events {
    pub fn new(duration: u64) -> Self {
        // チャネル送受信機生成
        let (tx, rx) = mpsc::channel();

        // 後でmoveするので送信機の情報をコピー
        let event_tx = tx.clone();

        // xxミリ秒間隔でキー受付
        let tick_rate = Duration::from_millis(duration);

        // スレッド生成 バックグラウンドでループ
        // 所有権をスレッド内にmove
        thread::spawn(move || {
            // 現在時間を経過時間を管理するために生成
            let mut last_tick = Instant::now();
            loop {
                // 経過時間の差を取得
                // Durationが0になることを意図して経過時間を記録し続ける
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                // Durationが0以外ならpoll
                if event::poll(timeout).unwrap() {
                    // キー入力をrxにsend
                    if let CEvent::Key(key) = event::read().unwrap() {
                        event_tx.send(Event::Input(key)).unwrap();
                    }
                }

                // 経過秒が200ミリ秒を超えたらtickを送信して経過秒をリセット
                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = event_tx.send(Event::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        // ループ中の送受信機を外だし
        Self { rx, _tx: tx }
    }

    pub fn next(&self) -> Result<Event<KeyEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}
