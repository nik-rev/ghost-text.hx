(define package-name 'ghost-text)
(define version "0.1.2")

(define dependencies '())

(define dylibs
  '((#:name "ghost_text"
     #:urls
     (
      (
       #:platform
       "x86_64-windows"
       #:url
       "https://github.com/nik-rev/ghost-text.hx/releases/download/v0.1.2/ghost_text.dll")
      (
       #:platform
       "x86_64-macos"
       #:url
       "https://github.com/nik-rev/ghost-text.hx/releases/download/v0.1.2/libghost_text.dylib")
      (
       #:platform
       "x86_64-linux"
       #:url
       "https://github.com/nik-rev/ghost-text.hx/releases/download/v0.1.2/libghost_text.so")))))
