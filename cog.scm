(define package-name 'ghost-text)
(define version "0.1.5")

(define dylib-name "ghost_text")
(define repo "nik-rev/ghost-text.hx")

(define dependencies '())

(define dylibs
  '((#:name dylib-name
     #:urls
     (
      (
       #:platform
       "x86_64-windows"
       #:url
       "https://github.com/" repo "/releases/download/v" version "/" dylib-name ".dll")
      (
       #:platform
       "x86_64-macos"
       #:url
       "https://github.com/" repo "/releases/download/v" version "/lib" dylib-name ".dylib")
      (
       #:platform
       "x86_64-linux"
       #:url
       "https://github.com/" repo "/releases/download/v" version "/lib" dylib-name ".so")))))
