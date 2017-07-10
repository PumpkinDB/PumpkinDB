( Cursor-related functionality )
CURSOR/SEEKLAST : ['key SET
                   key [] $SYSTEM/MAXKEYSIZE key LENGTH UINT/SUB 0xff PAD CONCAT 'padded SET
                   ( seek padded key )
                   DUP padded CURSOR/SEEK
                   ( if there's nothing, set cursor at the last item)
                   [TRUE] [DUP CURSOR/LAST] IFELSE
                   ( if we still can't position, it's an empty database )
                   NOT [NIP FALSE] [
                       ( if the key starts with the correct prefix, we are done )
                       DUP CURSOR/KEY key STARTSWITH?
                       [DROP TRUE]
                       ( otherwise, go to the previous key )
                       [DUP CURSOR/PREV SWAP CURSOR/KEY key STARTSWITH? AND]
                       IFELSE
                   ] IFELSE
                   ] EVAL/SCOPED.
CURSOR/DOWHILE : ['iterator SET 'closure SET 'c SET
                   [`c `closure EVAL [``c ``iterator EVAL] [FALSE] IFELSE] DOWHILE] EVAL/SCOPED.
CURSOR/DOWHILE-PREFIXED : ['closure SET 'prefix SET
                           CURSOR 'c SET
                           c prefix CURSOR/SEEK
                           [`c [DUP CURSOR/KEY ``prefix STARTSWITH?
                                [```closure EVAL] [DROP FALSE] IFELSE
                               ] 'CURSOR/NEXT CURSOR/DOWHILE] IF] EVAL/SCOPED.