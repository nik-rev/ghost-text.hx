(require "helix/ext.scm")
(require (prefix-in helix.static. "helix/static.scm"))
(require "helix/components.scm")
(require "helix/editor.scm")
(require-builtin helix/core/text as text::)
(require-builtin helix/core/static as static::)
(require-builtin helix/core/misc as misc::)
(require-builtin steel/json as json::)
(require-builtin steel/ffi)

(#%require-dylib "libghost_text"
  (only-in
    Server::new
    Server::start
    Server::init_logging
    REGISTER_HELIX_BUFFER
    Server::update
    Server::stop))

; Get contents of the current file as a `String`
(define (get-current-text)
  (text::rope->string
    (editor->text
      (editor->doc-id
        (editor-focus)))))

; set-current-selection-object! Selection

; Get a `Vec<(u32, u32)>` corresponding to a list of selections [from, to]
; for the current file.
(define (get-current-selection)
  (map
    static::range->span
    (static::selection->ranges
      (static::current-selection-object *helix.cx*))))

(Server::init_logging)

; This function is called from RUST
; to update contents of the current buffer
(define (update-document new-text)
  (log::info! "called this function")
  (hx.block-on-task
    (lambda ()
      (helix.static.select_all)
      (helix.static.delete_selection)
      (helix.static.insert_string new-text))))

(REGISTER_HELIX_BUFFER (function->ffi-function update-document))
(define server (Server::new))
(define is-server-running #false)

; Start the Ghost Text server
(define (ghost-text-start)
  (unless is-server-running
    (Server::start server)
    (set! is-server-running #true)
    "Started Ghost Text server"))

; Stop the Ghost Text server
(define (ghost-text-stop)
  (when is-server-running
    (Server::stop server)
    (set! is-server-running #false)
    "Killed Ghost Text server"))

; Update the Ghost Text server
(define (ghost-text-update)
  (when is-server-running
    (Server::update
      server
      (get-current-text)
      (get-current-selection))))

; Callback to pass to the hooks for updating the ghost text server
(define (update-if-running _)
  (when is-server-running
    (ghost-text-update)))

; When we run any command in Helix, send it to the Ghost Text server
(register-hook! "post-command" update-if-running)

; The above does not account for when we simply insert characters,
; in which case we *also* want to notify the Ghost Text server
(register-hook! "post-insert-char" update-if-running)
