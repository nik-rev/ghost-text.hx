(require "helix/components.scm")
(require "helix/editor.scm")
(require-builtin helix/core/text as text.)
(require-builtin helix/core/static as static.)

; Get a `Selection` of the current file
(define (get-current-selection) (static.current-selection-object))

; Get a `String` representing the text of the current file
(define (get-current-text)
  ; `rope->string` takes `Rope`
  ;   return `String`
  (text.rope->string
    ; `editor->text` takes DocumentId
    ;   return `Rope`
    (editor->text
      ; `editor->doc-id` takes `ViewId`
      ;   return `DocumentId`
      (editor->doc-id
        ; `editor-focus`
        ;   return `ViewId` of focused
        (editor-focus)))))

; This is where we want to send things to the GhostText server
(define (notify-ghost-text-server name-of-command)
  (log::info! (get-current-text)))

; When we run any command in Helix, send it to the Ghost Text server
(register-hook! "post-command" notify-ghost-text-server)
; The above does not account for when we simply insert characters,
; in which case we *also* want to notify the Ghost Text server
(register-hook! "post-insert-char" notify-ghost-text-server)
