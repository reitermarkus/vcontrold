use std::collections::HashMap;
use std::io::{self, Read, BufReader};
use std::fs::File;
use std::str::FromStr;
use std::path::{Path, PathBuf};

use serde_derive::*;
use serde_yaml;
use yaml_merge_keys;
