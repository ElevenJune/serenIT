pub struct Sound {
    name: String,
    path: String,
    volume: f32,
    //sink:Option<SinkHandle>
}

impl Sound {
    pub fn new(name: &str, path: &str, volume: f32) -> Self {
        Sound {
            name: name.to_string(),
            path: path.to_string(),
            volume,
            //sink: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    /*pub fn is_playing(&self) -> bool {
        match &self.sink {
            Some(sink) => sink.is_playing(),
            None => false,
        }
    }

    pub fn set_sink(&mut self, sink: SinkHandle) {
        self.sink = Some(sink);
    }

    pub fn remove_sink(&mut self) {
        self.sink = None;
    }*/
}
