;; Peter Norvig - Programming Challange from Erann Gat:
;; http://www.flownet.com/ron/papers/lisp-java/
;; Given a list of words and a list of phone numbers, find all the ways that
;; each phone number can be expressed as a list of words.
;; Run: (main "word-list-file-name" "phone-number-file-name")

(declaim (optimize (speed 3) (debug 0) (safety 0)))
(setq *block-compile-default* t)

(declaim (inline nth-digit char->digit digitp))

(declaim (ftype (function (simple-string (unsigned-byte 8)) string) nth-digit))
(defun nth-digit (digits i)
  "The i-th element of a character string of digits, as an integer 0 to 9."
  (ecase (char digits i)
    ((#\0) "0")
    ((#\1) "1")
    ((#\2) "2")
    ((#\3) "3")
    ((#\4) "4")
    ((#\5) "5")
    ((#\6) "6")
    ((#\7) "7")
    ((#\8) "8")
    ((#\9) "9")))

(defmacro scase (s &rest cases)
  (cons 'or (loop for c in cases
                  collect (list 'if (list 'string= s (car c)) (cadr c) ))))

(declaim (ftype (simple-string) boolean) digitp)
(defun digitp (s)
  (and
   (= 1 (length s))
   (scase s
          ("0" t)
          ("1" t)
          ("2" t)
          ("3" t)
          ("4" t)
          ("5" t)
          ("6" t)
          ("7" t)
          ("8" t)
          ("9" t))))

(declaim (ftype (function (base-char) simple-string) char->digit))
(defun char->digit (ch)
  "Convert a character to a digit according to the phone number rules."
  (ecase (char-downcase ch)
    ((#\e) "0")
    ((#\j #\n #\q) "1")
    ((#\r #\w #\x) "2")
    ((#\d #\s #\y) "3")
    ((#\f #\t) "4")
    ((#\a #\m) "5")
    ((#\c #\i #\v) "6")
    ((#\b #\k #\u) "7")
    ((#\l #\o #\p) "8")
    ((#\g #\h #\z) "9")))

(declaim (ftype (function (simple-string) simple-string)))
(defun word->number (word)
  "Translate a word (string) into a phone number, according to the rules."
  (let ((n ""))
    (loop for i from 0 below (length word)
          for ch = (char word i) do
          (when (alpha-char-p ch) (setq n (concatenate 'string n (char->digit ch)))))
    n))

(defglobal *dict* nil
  "A hash table mapping a phone number (integer) to a list of words from the
  input dictionary that produce that number.")

(declaim (ftype (function (string string (unsigned-byte 8) list))))
(defun print-translations (num digits &optional (start 0) (words nil))
  "Print each possible translation of NUM into a string of words.  DIGITS
  must be WORD with non-digits removed.  On recursive calls, START is the
  position in DIGITS at which to look for the next word, and WORDS is the list
  of words found for (subseq DIGITS 0 START).  So if START gets to the end of
  DIGITS, then we have a solution in WORDS.  Otherwise, for every prefix of
  DIGITS, look in the dictionary for word(s) that map to the value of the
  prefix (computed incrementally as N), and for each such word try to extend
  the solution with a recursive call.  There are two complications: (1) the
  rules say that in addition to dictionary words, you can use a single
  digit in the output, but not two digits in a row. Also (and this seems
  silly) you can't have a digit in a place where any word could appear.
  I handle this with the variable FOUND-WORD; if it is false after the loop,
  and the most recent word is not a digit, try a recursive call that pushes a
  digit. (2) The other complication is that the obvious way of mapping
  strings to integers would map R to 2 and ER to 02, which of course is
  the same integer as 2.  Therefore we prepend a 1 to every number, and R
  becomes 12 and ER becomes 102."
  (if (>= start (length digits))
      (format t "~a:~{ ~a~}~%" num (reverse words))
      (let ((next-iterations
              (do ((i start (1+ i)) ; var, initial value, increment per iteration
                   (n "")
                   (max (length digits))
                   (result nil))
                  ((>= i max) result) ; exit condition and return-value
                (setq n (concatenate 'string n (nth-digit digits i)))
                (let ((next-words (gethash n *dict*)))
                  (when next-words (push (list (1+ i) next-words) result))))))
        (if next-iterations
            (loop for (i next-words) in next-iterations do
              (loop for word in next-words do
                (print-translations num digits i (cons word words))))
            (when (not (digitp (first words)))
              (print-translations num digits (+ start 1)
                                  (cons (nth-digit digits start) words)))))))

(defun load-dictionary (file size)
  "Create a hashtable from the file of words (one per line).  Takes a hint
  for the initial hashtable size.  Each key is the phone number for a word;
  each value is a list of words with that phone number."
  (let ((table (make-hash-table :test #'equal :size size)))
    (with-open-file (in file)
      (loop for word = (read-line in nil) while word do
        (push word (gethash (word->number word) table))))
    table))


(defun main (&optional (dict "tests/words.txt") (nums "tests/numbers.txt") (dict-size 100))
  "Read the input file ¨DICT and load it into *dict*.  Then for each line in
  NUMS, print all the translations of the number into a sequence of words,
  according to the rules of translation."
  (setf *dict* (load-dictionary dict dict-size))
  (with-open-file (in nums)
    (loop for num = (read-line in nil) while num do
          (print-translations num (remove-if-not #'digit-char-p num)))))

(apply #'main (cdr sb-ext:*posix-argv*))
