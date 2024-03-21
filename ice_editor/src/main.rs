use std::{
    io,
    path::{
        Path, PathBuf
    }, sync::Arc
};
use tokio::fs;
use iced::{
        executor, widget::{
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
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    New,
    Open,
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

                Command::none()
            }
            Message::Open => Command::perform(pick_file(), Message::FileOpened),
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();

                Command::none()
            },
            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&content);

                Command::none()
            },
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
        
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![button("New").on_press(Message::New), button("Open").on_press(Message::Open)].spacing(5);
        
        let input = text_editor(&self.content).on_edit(Message::Edit);

        
        let status_bar = {
            let file_path = match self.path.as_deref().and_then(Path::to_str) {
                Some(path) => text(path).size(14),
                None => text(""),
            };
            
            let position = {
                let (line, column) = self.content.cursor_position();
                
                text(format!("{}:{}", line + 1, column + 1))
            };
            row![file_path, horizontal_space(Length::Fill), position];
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

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IO(io::ErrorKind)
}