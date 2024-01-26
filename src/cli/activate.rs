use std::path::{Path, PathBuf};

use eyre::Result;

use crate::file::touch_dir;
use crate::shell::{get_shell, Shell, ShellType};
use crate::{dirs, env};

/// Initializes mise in the current shell session
///
/// This should go into your shell's rc file.
/// Otherwise, it will only take effect in the current session.
/// (e.g. ~/.zshrc, ~/.bashrc)
///
/// This is only intended to be used in interactive sessions, not scripts.
/// mise is only capable of updating PATH when the prompt is displayed to the user.
/// For non-interactive use-cases, use shims instead.
///
/// Typically this can be added with something like the following:
///
///     echo 'eval "$(mise activate)"' >> ~/.zshrc
///
/// However, this requires that "mise" is in your PATH. If it is not, you need to
/// specify the full path like this:
///
///     echo 'eval "$(/path/to/mise activate)"' >> ~/.zshrc
#[derive(Debug, clap::Args)]
#[clap(verbatim_doc_comment, after_long_help = AFTER_LONG_HELP)]
pub struct Activate {
    /// Shell type to generate the script for
    #[clap(long, short, hide = true)]
    shell: Option<ShellType>,

    /// Shell type to generate the script for
    #[clap()]
    shell_type: Option<ShellType>,

    /// Show "mise: <PLUGIN>@<VERSION>" message when changing directories
    #[clap(long, hide = true)]
    status: bool,

    /// Use shims instead of modifying PATH
    /// Effectively the same as:
    ///     PATH="$HOME/.local/share/mise/shims:$PATH"
    #[clap(long, verbatim_doc_comment)]
    shims: bool,

    /// Suppress non-error messages
    #[clap(long, short)]
    quiet: bool,
}

impl Activate {
    pub fn run(self) -> Result<()> {
        let shell = get_shell(self.shell_type.or(self.shell))
            .expect("no shell provided. Run `mise activate zsh` or similar");

        // touch ROOT to allow hook-env to run
        let _ = touch_dir(&dirs::DATA);

        let mise_bin = if cfg!(target_os = "linux") {
            // linux dereferences symlinks, so use argv0 instead
            PathBuf::from(&*env::ARGV0)
        } else {
            env::MISE_BIN.clone()
        };
        let mut flags = vec![];
        if self.quiet {
            flags.push(" --quiet");
        }
        if self.status {
            flags.push(" --status");
        }
        let output = match self.shims {
            true => self.activate_shims(shell.as_ref(), &mise_bin),
            false => shell.activate(&mise_bin, flags.join("")),
        };
        miseprint!("{output}");

        Ok(())
    }

    fn activate_shims(&self, shell: &dyn Shell, exe: &Path) -> String {
        let exe_dir = exe.parent().unwrap().to_path_buf();
        shell.prepend_path(&[exe_dir, dirs::SHIMS.clone()])
    }
}

static AFTER_LONG_HELP: &str = color_print::cstr!(
    r#"<bold><underline>Examples:</underline></bold>
  $ <bold>eval "$(mise activate bash)"</bold>
  $ <bold>eval "$(mise activate zsh)"</bold>
  $ <bold>mise activate fish | source</bold>
  $ <bold>execx($(mise activate xonsh))</bold>
"#
);
