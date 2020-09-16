\ A table is:  #lastentry test-xt
 \ An entry is: matcher previous xt

 : >xt [ 2 cells ] literal + ;
 : >link cell+ ;

 \ search the links for n as long as xt returns true.
 : (table) ( n nextlink test-xt)
   >r
   begin
 	?dup if  r> drop s" Not found " type -1 throw then
 	2dup @ r@ execute while
 	>link @
   repeat
   r> drop
   nip
   >xt @ execute
 ;

 : fn@ cell+ @ ;

 : table: ( xt <name> -- baseaddr prevdummy)
   create here  0 ,
   swap ,
   0
 does> ( n addr)
   dup
   @  ( n addr last )
   swap
   fn@
   (table)
 ;

 : extend: ( <tableName> -- baseaddr prev)
   ' >pf @  ( baseaddr)
   dup @
 ;
