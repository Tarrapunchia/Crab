use std::{
    io,
    path::{
        Path, PathBuf
    }, sync::Arc
};
use tokio::fs;
use iced::{
        color, executor, widget::{
            button,
            column,
            container,
            horizontal_space,
            row,
            text,
            text_editor,
        }, Application, Command, Element, Length, Settings, Theme
    };

fn main() -> iced::Result{
    Editor::run(Settings::default())
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    New,
    Save,
    FileSaved(Result<(), Error>),
}

impl Application for Editor {
    type Message = Message;
    type Executor = executor::Default; // default engine
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) { // app initial state
        (Self {
            path: None,
            content: text_editor::Content::new(),
            error: None,
        },
            Command::perform(
                load_file(default_file()),
          Message::FileOpened,
            ),
        )
    }

    fn title(&self) -> String { // title of window app
        String::from("Crab!")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
                self.error = None; // clear error when re-editing
                Command::none()
            }
            Message::Open => Command::perform(pick_file(), Message::FileOpened),
            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&content);

                Command::none()
            },
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();

                Command::none()
            },
            Message::Save => {
                let text = self.content.text();
                let path = self.path.clone();
                Command::perform(save_file(path, text), Message::FileSaved)
            },
            Message::FileSaved(Ok(())) => {
                self.error = None;

                Command::none()
            },
            Message::FileSaved(Err(error)) => {
                self.error = Some(error);

                Command::none()
            },
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
        
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            button("New").on_press(Message::New),
            button("Open").on_press(Message::Open),
            button("Save").on_press(Message::Save)]
            .spacing(5);
        
        let input = text_editor(&self.content).on_edit(Message::Edit);

        
        let status_bar = {
            let status = if let Some(Error::IO(error)) = self.error.as_ref() {
                text(error.to_string())
                } else {
                match self.path.as_deref().and_then(Path::to_str) {
                Some(path) => text(path).size(14),
                None => text("New File"),
                }
            };
            
            let position = {
                let (line, column) = self.content.cursor_position();
                
                text(format!("{}:{}", line + 1, column + 1))
            };
            row![status, horizontal_space(Length::Fill), position]
        };
        container(column![controls, input, status_bar].spacing(5)).padding(5).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
    
}

/// set default file
fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}


/// pick a file
async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;
    
    load_file(handle.path().to_owned()).await
}
 
/// file loader
async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let content = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IO)?;

    Ok((path, content))
}

/// file saver
async fn save_file(path: Option<PathBuf>, text: String) -> Result<(), Error> {
    // if we have a path we save to it, else we ask for a new path
    let path = if let Some(path) = path { path } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file name...")
            .save_file()
            .await
            .ok_or(Error::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };

    tokio::fs::write(&path, &text)
        .await
        .map_err(|error| Error::IO(error.kind()))
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IO(io::ErrorKind)
}
