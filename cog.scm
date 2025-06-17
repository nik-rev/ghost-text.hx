(define package-name 'ghost-text)
(define version "0.1.2")

(define dependencies '())
(define dylibs '((#:name "ghost-text")))

(define dylibs
  '((#:name "ghost-text"
     #:urls
     (
      (
       #:platform
       "windows"
       #:url
       "https://github.com/nik-rev/ghost-text.hx/releases/download/v0.1.2/ghost_text.dll")
      (
       #:platform
       "macos"
       #:url
       "https://github.com/nik-rev/ghost-text.hx/releases/download/v0.1.2/libghost_text.dylib")
      (
       #:platform
       "linux"
       #:url
       "https://github.com/nik-rev/ghost-text.hx/releases/download/v0.1.2/libghost_text.so")))))
