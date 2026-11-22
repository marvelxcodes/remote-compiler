set -gx REMOTE_CARGO_HOST user@build-server

function cargo --wraps cargo --description 'Build and run on the remote compilation server'
    if contains -- "$argv[1]" run build; and type -q cargo-remote
        cargo-remote $argv
    else
        command cargo $argv
    end
end
