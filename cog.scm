(require "helix/components.scm")
(require "helix/editor.scm")
(require-builtin helix/core/text as text::)
(require-builtin helix/core/static as static::)
(require-builtin steel/json as json::)

(#%require-dylib "libghost_text"
  (only-in
    Server::new
    Server::start
    Server::init_logging
    Server::update
    Server::stop))

; Get contents of the current file as a `String`
(define (get-current-text)
  (text::rope->string
    (editor->text
      (editor->doc-id
        (editor-focus)))))

; Get a `Vec<(u32, u32)>` corresponding to a list of selections [from, to]
; for the current file.
(define (get-current-selection)
  (map
    static::range->span
    (static::selection->ranges
      (static::current-selection-object *helix.cx*))))

(Server::init_logging)

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
