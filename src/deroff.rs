
// #[test]
// fn test_deroff() {
//     // let path = "./fixtures/docker-rmi.1".to_owned();
//     // let path = "./fixtures/qelectrotech.1".to_owned();
//     let path = "./fixtures/mlterm.1".to_owned();
//     // let path = "./sstest.1".to_owned();
//     // deroff_files(&[path]);
//     let mut d = Deroffer::new();
//     // d.deroff(q.to_owned());
//     use std::io::Write;
//     let mut output = std::fs::File::create("./q.deroff").unwrap();
//     eprintln!("{:?}", d.output);
// }

// #[cfg(test)]
// mod benches {
//     extern crate test;
//     use super::*;
//     use test::Bencher;
//     #[bench]
//     fn bench_deroff(b: &mut Bencher) {
//         b.iter(|| {
//             deroff_files(&["./fixtures/qelectrotech.1".to_owned()]);
//         })
//     }
// }

/// A translation of https://github.com/fish-shell/fish-shell/blob/e7bfd1d71ca54df726a4f1ea14bd6b0957b75752/share/tools/deroff.py
// """ Deroff.py, ported to Python from the venerable deroff.c """
use libflate::gzip::Decoder;
use regex::Regex;

use crate::util::{maketrans, translate};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

const SKIP_LISTS: bool = false;
const SKIP_HEADERS: bool = false;

enum TblState {
    Options,
    Format,
    Data,
}

// class Deroffer:
struct Deroffer {
    g_re_word: &'static Regex,
    g_re_number: &'static Regex,
    g_re_not_backslash_or_whitespace: &'static Regex,
    g_re_newline_collapse: &'static Regex,
    g_re_font: &'static Regex,

    reg_table: HashMap<String, String>,
    tr_from: String,
    tr_to: String,
    tr: Option<HashMap<char, char>>,
    specletter: bool,
    refer: bool,
    r#macro: bool,
    nobody: bool,
    inlist: bool,
    inheader: bool,
    pic: bool,
    tbl: bool,
    tblstate: TblState,
    tblTab: String,
    eqn: bool,
    output: String,
    name: String,

    s: String, // This is not explicitly defined in python code
}

impl Deroffer {
    fn new() -> Deroffer {
        Deroffer {
            g_re_word: crate::regex!(r##"[a-zA-Z_]+"##),
            g_re_number: crate::regex!(r##"[+-]?\d+"##),
            // sequence of not backslash or whitespace
            g_re_not_backslash_or_whitespace: crate::regex!(r##"[^ \t\n\r\f\v\\]+"##),
            g_re_newline_collapse: crate::regex!(r##"\n{3,}"##),
            g_re_font: crate::regex!(
                r##"(?x)\\f(     # Starts with backslash f
                    (\(\S{2})  | # Open paren, then two printable chars
                    (\[\S*?\]) | # Open bracket, zero or more printable characters, then close bracket
                    \S)          # Any printable character
                   "##),

            reg_table: HashMap::new(),
            tr_from: String::new(),
            tr_to: String::new(),
            tr: None,
            specletter: false,
            refer: false,
            r#macro: false,
            nobody: false,
            inlist: false,
            inheader: false,
            pic: false,
            tbl: false,
            tblstate: TblState::Options,
            tblTab: String::new(),
            eqn: false,
            output: String::new(),
            name: String::new(),

            s: String::new(), // This is not explicitly defined in python code
        }
    }

    fn get_output(&self, output: &[u8]) -> Result<String, String> {
        let s = String::from_utf8(output.into())
            .map_err(|err| format!("Bad bad bad (bad utf8)! {}", err))?;
        Ok(self.g_re_newline_collapse.replace_all(&s, "\n").into())
    }

    // for the moment, return small strings, until we figure out what
    // it should really be doing
    fn g_specs_specletter(key: &str) -> Option<&'static str> {
        Some(match key {
            // Output composed latin1 letters
            "-D" => &"Ð",
            "Sd" => &"ð",
            "Tp" => &"þ",
            "TP" => &"Þ",
            "AE" => &"Æ",
            "ae" => &"æ",
            "OE" => &"OE",
            "oe" => &"oe",
            ":a" => &"ä",
            ":A" => &"Ä",
            ":e" => &"ë",
            ":E" => &"Ë",
            ":i" => &"ï",
            ":I" => &"Ï",
            ":o" => &"ö",
            ":O" => &"Ö",
            ":u" => &"ü",
            ":U" => &"Ü",
            ":y" => &"ÿ",
            "ss" => &"ß",
            "'A" => &"Á",
            "'E" => &"É",
            "'I" => &"Í",
            "'O" => &"Ó",
            "'U" => &"Ú",
            "'Y" => &"Ý",
            "'a" => &"á",
            "'e" => &"é",
            "'i" => &"í",
            "'o" => &"ó",
            "'u" => &"ú",
            "'y" => &"ý",
            "^A" => &"Â",
            "^E" => &"Ê",
            "^I" => &"Î",
            "^O" => &"Ô",
            "^U" => &"Û",
            "^a" => &"â",
            "^e" => &"ê",
            "^i" => &"î",
            "^o" => &"ô",
            "^u" => &"û",
            "`A" => &"À",
            "`E" => &"È",
            "`I" => &"Ì",
            "`O" => &"Ò",
            "`U" => &"Ù",
            "`a" => &"à",
            "`e" => &"è",
            "`i" => &"ì",
            "`o" => &"ò",
            "`u" => &"ù",
            "~A" => &"Ã",
            "~N" => &"Ñ",
            "~O" => &"Õ",
            "~a" => &"ã",
            "~n" => &"ñ",
            "~o" => &"õ",
            ",C" => &"Ç",
            ",c" => &"ç",
            "/l" => &"/l",
            "/L" => &"/L",
            "/o" => &"ø",
            "/O" => &"Ø",
            "oA" => &"Å",
            "oa" => &"å",

            // Ligatures
            "fi" => &"fi",
            "ff" => &"ff",
            "fl" => &"fl",
            "Fi" => &"ffi",
            "Ff" => &"fff",
            "Fl" => &"ffl",
            _ => return None,
        })
    }

    // Much like the above, return small strings for now until we know what
    // we might actually want to do
    fn g_specs(key: &str) -> Option<&'static str> {
        Some(match key {
            "mi" => &"-",
            "en" => &"-",
            "hy" => &"-",
            "em" => &"--",
            "lq" => &"“",
            "rq" => &"”",
            "Bq" => &",,",
            "oq" => &"`",
            "cq" => &"'",
            "aq" => &"'",
            "dq" => &"\"",
            "or" => &"|",
            "at" => &"@",
            "sh" => &"#",
            // For the moment, faithfully mimic the behavior of the Python script,
            // even though it might seem that &"€" is a more appropriate result here
            "Eu" => &"¤",
            "eu" => &"¤",
            "Do" => &"$",
            "ct" => &"¢",
            "Fo" => &"«",
            "Fc" => &"»",
            "fo" => &"<",
            "fc" => &">",
            "r!" => &"¡",
            "r?" => &"¿",
            "Of" => &"ª",
            "Om" => &"º",
            "pc" => &"·",
            "S1" => &"¹",
            "S2" => &"²",
            "S3" => &"³",
            "<-" => &"<-",
            "->" => &"->",
            "<>" => &"<->",
            "ua" => &"^",
            "da" => &"v",
            "lA" => &"<=",
            "rA" => &"=>",
            "hA" => &"<=>",
            "uA" => &"^^",
            "dA" => &"vv",
            "ba" => &"|",
            "bb" => &"|",
            "br" => &"|",
            "bv" => &"|",
            "ru" => &"_",
            "ul" => &"_",
            "ci" => &"O",
            "bu" => &"o",
            "co" => &"©",
            "rg" => &"®",
            "tm" => &"(TM)",
            "dd" => &"||",
            "dg" => &"|",
            "ps" => &"¶",
            "sc" => &"§",
            "de" => &"°",
            "%0" => &"0/00",
            "14" => &"¼",
            "12" => &"½",
            "34" => &"¾",
            "f/" => &"/",
            "sl" => &"/",
            "rs" => &"\\",
            "sq" => &"[]",
            "fm" => &"'",
            "ha" => &"^",
            "ti" => &"~",
            "lB" => &"[",
            "rB" => &"]",
            "lC" => &"{",
            "rC" => &"}",
            "la" => &"<",
            "ra" => &">",
            "lh" => &"<=",
            "rh" => &"=>",
            "tf" => &"therefore",
            "~~" => &"~~",
            "~=" => &"~=",
            "!=" => &"!=",
            "**" => &"*",
            "+-" => &"±",
            "<=" => &"<=",
            "==" => &"==",
            "=~" => &"=~",
            ">=" => &">=",
            "AN" => &"\\/",
            "OR" => &"/\\",
            "no" => &"¬",
            "te" => &"there exists",
            "fa" => &"for all",
            "Ah" => &"aleph",
            "Im" => &"imaginary",
            "Re" => &"real",
            "if" => &"infinity",
            "md" => &"·",
            "mo" => &"member of",
            "mu" => &"×",
            "nm" => &"not member of",
            "pl" => &"+",
            "eq" => &"=",
            "pt" => &"oc",
            "pp" => &"perpendicular",
            "sb" => &"(=",
            "sp" => &"=)",
            "ib" => &"(-",
            "ip" => &"-)",
            "ap" => &"~",
            "is" => &"I",
            "sr" => &"root",
            "pd" => &"d",
            "c*" => &"(x)",
            "c+" => &"(+)",
            "ca" => &"cap",
            "cu" => &"U",
            "di" => &"÷",
            "gr" => &"V",
            "es" => &"{}",
            "CR" => &"_|",
            "st" => &"such that",
            "/_" => &"/_",
            "lz" => &"<>",
            "an" => &"-",

            // Output Greek
            "*A" => &"Alpha",
            "*B" => &"Beta",
            "*C" => &"Xi",
            "*D" => &"Delta",
            "*E" => &"Epsilon",
            "*F" => &"Phi",
            "*G" => &"Gamma",
            "*H" => &"Theta",
            "*I" => &"Iota",
            "*K" => &"Kappa",
            "*L" => &"Lambda",
            "*M" => &"Mu",
            "*N" => &"Nu",
            "*O" => &"Omicron",
            "*P" => &"Pi",
            "*Q" => &"Psi",
            "*R" => &"Rho",
            "*S" => &"Sigma",
            "*T" => &"Tau",
            "*U" => &"Upsilon",
            "*W" => &"Omega",
            "*X" => &"Chi",
            "*Y" => &"Eta",
            "*Z" => &"Zeta",
            "*a" => &"alpha",
            "*b" => &"beta",
            "*c" => &"xi",
            "*d" => &"delta",
            "*e" => &"epsilon",
            "*f" => &"phi",
            "+f" => &"phi",
            "*g" => &"gamma",
            "*h" => &"theta",
            "+h" => &"theta",
            "*i" => &"iota",
            "*k" => &"kappa",
            "*l" => &"lambda",
            "*m" => &"µ",
            "*n" => &"nu",
            "*o" => &"omicron",
            "*p" => &"pi",
            "+p" => &"omega",
            "*q" => &"psi",
            "*r" => &"rho",
            "*s" => &"sigma",
            "*t" => &"tau",
            "*u" => &"upsilon",
            "*w" => &"omega",
            "*x" => &"chi",
            "*y" => &"eta",
            "*z" => &"zeta",
            "ts" => &"sigma",
            _ => return None,
        })
    }

    fn skip_char(&mut self, amount: usize) {
        self.s = self.s.get(amount..).unwrap_or("").to_owned();
    }

    /* fn skip_char<'a>(&self, s: &'a str, amount: Option<usize>) -> &'a str {
        let amount = amount.unwrap_or(1);
        s.get(amount..).unwrap_or("")
    } */

    fn skip_leading_whitespace(&mut self) {
        self.s = self.s.trim_start().to_owned();
    }

    /* fn skip_leading_whitespace<'a>(&self, s: &'a str) -> &'a str {
        s.trim_start()
    } */

    fn str_at(&mut self, idx: usize) -> &str {
        let s = &self.s;
        s.char_indices()
            .skip(idx)
            .next()
            .map(|(i, c)| &s[i..(i + c.len_utf8())])
            .unwrap_or("")
    }

    /* fn str_at(string: &str, idx: usize) -> &str {
        // Note: If we don't care about strings with multi-byte chars, the
        // following would suffice:
        // s.get(idx..idx + 1).unwrap_or("")
        //
        // Note: We're not yet sure whether our roff inputs will generally be
        // ASCII or UTF-8. If they are ASCII (and can be treated as containing
        // only single-byte characters), it would be faster to just use `get()`
        string
            .char_indices()
            .nth(idx)
            .map(|(idx, charr)| &string[idx..(idx + charr.len_utf8())]) // Okay to directly index based on idx/charr construction.
            .unwrap_or_default()
    } */

    fn is_white(&self, idx: usize) -> bool {
        self.s // String
            .chars() // Chars
            .nth(idx) // Option<char>
            .unwrap_or('a') // char
            .is_whitespace() // bool
    }

    /* fn is_white<'a>(s: &'a str, idx: usize) -> bool {
        s
          .chars()  // Chars
          .nth(idx) // Option<char>
          .unwrap_or('a') // char
          .is_whitespace() // bool
    } */

    // This is also known as `prch` apparently
    fn not_whitespace(&self, idx: usize) -> bool {
        !" \t\n".contains(self.s.get(idx..idx + 1).unwrap_or_default())
    }

    /* fn not_whitespace(s: &str, idx: usize) -> bool {
        // # Note that this return False for the empty string (idx >= len(self.s))
        // ch = self.s[idx:idx+1]
        // return ch not in ' \t\n'
        // TODO Investigate checking for ASCII whitespace after mvp
        s.get(idx..(idx + 1))
            .map(|string| " \t\n".contains(string))
            .unwrap_or_default()
    } */

    // Replaces the g_macro_dict lookup in the Python code
    fn g_macro_dispatch(&mut self, s: &str) -> bool {
        match s {
            "SH" => self.macro_sh(),
            "SS" => self.macro_ss_ip(),
            "IP" => self.macro_ss_ip(),
            "H " => self.macro_ss_ip(),
            "I " => self.macro_i_ir(),
            "IR" => self.macro_i_ir(),
            "IB" => self.macro_i_ir(),
            "B " => self.macro_i_ir(),
            "BR" => self.macro_i_ir(),
            "BI" => self.macro_i_ir(),
            "R " => self.macro_i_ir(),
            "RB" => self.macro_i_ir(),
            "RI" => self.macro_i_ir(),
            "AB" => self.macro_i_ir(),
            "Nm" => self.macro_nm(),
            "] " => self.macro_close_bracket(),
            "PS" => self.macro_ps(),
            "PE" => self.macro_pe(),
            "TS" => self.macro_ts(),
            "T&" => self.macro_t_and(),
            "TE" => self.macro_te(),
            "EQ" => self.macro_eq(),
            "EN" => self.macro_en(),
            "R1" => self.macro_r1(),
            "R2" => self.macro_r2(),
            "de" => self.macro_de(),
            "BL" => self.macro_bl_vl(),
            "VL" => self.macro_bl_vl(),
            "AL" => self.macro_bl_vl(),
            "LB" => self.macro_bl_vl(),
            "RL" => self.macro_bl_vl(),
            "ML" => self.macro_bl_vl(),
            "DL" => self.macro_bl_vl(),
            "BV" => self.macro_bv(),
            "LE" => self.macro_le(),
            "LP" => self.macro_lp_pp(),
            "PP" => self.macro_lp_pp(),
            "P\n" => self.macro_lp_pp(),
            "ds" => self.macro_ds(),
            "so" => self.macro_so_nx(),
            "nx" => self.macro_so_nx(),
            "tr" => self.macro_tr(),
            "sp" => self.macro_sp(),
            _ => self.macro_other(),
        }
    }

    // Done by Kevin, not merged >:'(
    fn macro_sh(&mut self) -> bool {
        for header_str in [" SYNOPSIS", " \"SYNOPSIS", " ‹BERSICHT", " \"‹BERSICHT"].iter() {
            if self.s[3..].starts_with(header_str) {
                self.inheader = true;
                return true;
            }
        }

        self.inheader = false;
        self.nobody = true;
        false
    }

    // Done by Kevin, not merged >:'(
    fn macro_ss_ip(&mut self) -> bool {
        self.nobody = true;
        false
    }

    // Done by Kevin, not merged >:'(
    fn macro_i_ir(&mut self) -> bool {
        // why does this exist
        false
    }

    // Done by Kevin, not merged >:'(
    fn macro_nm(&mut self) -> bool {
        if self.s == "Nm\n" {
            self.condputs(self.name.clone().as_str());
        } else {
            let mut s = self.s[3..].trim().to_owned();
            s.push(' ');
            self.name = s;
        }

        true
    }

    // Done by Kevin, not merged >:'(
    fn macro_close_bracket(&mut self) -> bool {
        self.refer = false;
        false
    }

    // Done by Kevin, not merged >:'(
    fn macro_ps(&mut self) -> bool {
        if self.is_white(2) {
            self.pic = true
        }
        self.condputs("\n");
        true
    }

    fn macro_pe(&mut self) -> bool {
        /*
        def macro_pe(self):
            if self.is_white(2):
                self.pic = False
            self.condputs("\n")
            return True
        */
        if self.is_white(2) {
            self.pic = false
        }
        self.condputs("\n");
        true
    }

    fn macro_ts(&mut self) -> bool {
        /*
        def macro_ts(self):
            if self.is_white(2):
                self.tbl, self.tblstate = True, self.OPTIONS
            self.condputs("\n")
            return True
        */

        if self.is_white(2) {
            self.tbl = true;
            self.tblstate = TblState::Options;
        }

        self.condputs("\n");
        true
    }

    fn macro_t_and(&mut self) -> bool {
        /*
        def macro_t_and(self):
            if self.is_white(2):
                self.tbl, self.tblstate = True, self.FORMAT
            self.condputs("\n")
            return True
        */

        if self.is_white(2) {
            self.tbl = true;
            self.tblstate = TblState::Format;
        }

        self.condputs("\n");
        true
    }

    fn macro_te(&mut self) -> bool {
        /*
        def macro_te(self):
            if self.is_white(2):
                self.tbl = False
            self.condputs("\n")
            return True
        */

        if self.is_white(2) {
            self.tbl = false
        }

        self.condputs("\n");
        true
    }

    fn macro_eq(&mut self) -> bool {
        /*
        def macro_eq(self):
            if self.is_white(2):
                self.eqn = True
            self.condputs("\n")
            return True
        */

        if self.is_white(2) {
            self.eqn = true
        }

        self.condputs("\n");
        true
    }

    fn macro_en(&mut self) -> bool {
        /*
        def macro_en(self):
            if self.is_white(2):
                self.eqn = False
            self.condputs("\n")
            return True
        */

        if self.is_white(2) {
            self.eqn = false
        }

        self.condputs("\n");
        true
    }

    fn macro_r1(&mut self) -> bool {
        /*
        def macro_r1(self):
            if self.is_white(2):
                self.refer2 = True
            self.condputs("\n")
            return True
        */

        // NOTE: self.refer2 is never used in the python source, so this and macro_r2 are
        // pretty much worthless
        // if Self::is_white(self.s.as_str(), 2) {
        //     self.refer2 = true;
        // }
        self.condputs("\n");
        true
    }

    fn macro_r2(&mut self) -> bool {
        /*
            def macro_r2(self):
            if self.is_white(2):
                self.refer2 = False
            self.condputs("\n")
            return True
        */

        // if Self::is_white(self.s.as_str(), 2) {
        //     NOTE: See macro_r1
        //     self.refer2 = false;
        // }
        self.condputs("\n");
        true
    }

    fn macro_de(&mut self) -> bool {
        /*
        def macro_de(self):
            macro = True
            self.condputs("\n")
            return True
        */
        self.r#macro = true;
        self.condputs("\n");
        true
    }

    fn macro_bl_vl(&mut self) -> bool {
        /*
        def macro_bl_vl(self):
            if self.is_white(2):
                self.inlist = True
            self.condputs("\n")
            return True
        */

        if self.is_white(2) {
            self.inlist = true
        }
        self.condputs("\n");
        true
    }

    fn macro_bv(&mut self) -> bool {
        /*
        def macro_bv(self):
            if self.str_at(2) == "L" and self.white(self.str_at(3)):
                self.inlist = True
            self.condputs("\n")
            return True
        */

        /*
        `self.white` doesn't exist in the source, and the argument type is wrong
        for `self.is_white`, so I don't know what function its supposed to be
        if self.str_at(2) == "L" and self.white(self.str_at(3)):
            self.inlist = true
        } */
        self.condputs("\n");
        true
    }

    fn macro_le(&mut self) -> bool {
        /*
        def macro_le(self):
            if self.is_white(2):
                self.inlist = False
            self.condputs("\n")
            return True
        */
        if self.is_white(2) {
            self.inlist = false;
        }
        self.condputs("\n");
        true
    }

    fn macro_lp_pp(&mut self) -> bool {
        /*
        def macro_lp_pp(self):
            self.condputs("\n")
            return True
        */
        self.condputs("\n");
        true
    }

    fn macro_ds(&mut self) -> bool {
        /*
        def macro_ds(self):
            self.skip_char(2)
            self.skip_leading_whitespace()
            if self.str_at(0):
                # Split at whitespace
                comps = self.s.split(None, 2)
                if len(comps) == 2:
                    name, value = comps
                    value = value.rstrip()
                    self.reg_table[name] = value
            self.condputs("\n")
            return True
        */

        self.skip_char(2);
        self.skip_leading_whitespace();

        if !self.str_at(0).is_empty() {
            let comps: Vec<String> = self.s.splitn(2, " ").map(|s| s.to_owned()).collect();

            if comps.len() == 2 {
                let name: String = comps.get(0).unwrap().to_owned();
                /*
                This is a reminder to google stuff before you go implementing stuff badly

                // This is horrible I know but it's meant to do string.rstrip()
                // If you can think of a better way I am more than willing to switch it
                let value: String = comps
                            .get(1)
                            .unwrap() // This is safe (len 2)
                            .chars()
                            .rev() // reverse the string to get the right side
                            .skip_while(|c| c.is_whitespace()) // Skip any whitespace
                            .collect::<String>() // collect it
                            .chars()
                            .rev() // make the string face the right way
                            .collect(); // put it back in a String
                // A note on the badness of this code,
                // The reason for `.collect().rev().chars().collect()` exists is
                // `skip_while` returns a `SkipWhile` which doesnt impl `DoubleEndedIterator`
                // which is required for `rev` :( */

                let value = comps.get(1).unwrap().as_str().trim_end().to_owned();

                self.reg_table.insert(name, value);
            }
        }

        self.condputs("\n");
        true
    }

    // Done by Kevin, not merged >:'(
    fn macro_so_nx(&mut self) -> bool {
        /*  # We always ignore include directives
        # deroff.c for some reason allowed this to fall through to the 'tr' case
        # I think that was just a bug so I won't replicate it */
        true
    }

    // Done by Anders, not merged >:'(
    fn macro_tr(&mut self) -> bool {
        let s = &self.s.clone();
        self.skip_char(2);
        self.skip_leading_whitespace();
        while !s.is_empty() && &s[0..=0] != "\n" {
            let c = &s[0..=0];
            let mut ns = &s[1..=1];
            self.skip_char(2);
            if ns.is_empty() || ns == "\n" {
                ns = " ";
            }

            self.tr_from.push_str(c);
            self.tr_to.push_str(ns);
        }

        // Update our table, then swap in the slower tr-savvy condputs
        self.tr = Some(maketrans(self.tr_from.as_str(), self.tr_to.as_str()));
        true
    }

    fn macro_sp(&mut self) -> bool {
        /*
        def macro_sp(self):
            self.condputs("\n")
            return True
        */

        self.condputs("\n");
        true
    }

    fn macro_other(&mut self) -> bool {
        /*
        def macro_other(self):
            self.condputs("\n")
            return True
        */

        self.condputs("\n");
        true
    }

    /// `condputs` (cond)itionally (puts) `s` into `self.output`
    /// if `self.tr` is set, instead of putting `s` into `self.output` directly,
    /// it `translate`s it using the set translation table and puts the result
    /// into `self.output`
    fn condputs(&mut self, s: &str) {
        let is_special = {
            self.pic
                || self.eqn
                || self.refer
                || self.r#macro
                || (self.inlist && self.inheader)
                || (SKIP_HEADERS && SKIP_LISTS)
        };

        if !is_special {
            if let Some(table) = self.tr.clone() {
                self.output.push_str(translate(s.into(), table).as_str());
            } else {
                self.output.push_str(s);
            }
        }
    }

    fn number(&mut self) -> bool {
        match self.g_re_number.find(&self.s.clone()) {
            Some(m) => {
                self.condputs(m.as_str());
                self.skip_char(m.end());
                true
            }
            None => false,
        }
    }

    // Thanks Steve, I only modified it a little, like, completely
    fn esc_char(&mut self) -> bool {
        self.s
            .clone()
            .get(0..1)
            .and_then(|ch| {
                if ch == "\\" {
                    Some(self.esc_char_backslash())
                } else {
                    Some(self.word() || self.number())
                }
            })
            .unwrap_or(false)
    }

    fn quoted_arg(&mut self) -> bool {
        if self.str_at(0) == "\"" {
            self.skip_char(1);
            while !self.s.is_empty() && self.str_at(0) != "\"" {
                if !self.esc_char() {
                    // There's another check for s being empty here, but it "should" be safe w/o it
                    self.condputs(&self.s.clone()[0..=0]);
                    self.skip_char(1);
                }
            }
            true
        } else {
            false
        }
    }

    fn request_or_macro(&mut self) -> bool {
        self.skip_char(1);

        let s0 = self.str_at(1);

        match s0 {
            "\\" => {
                if self.str_at(1) == "\"" {
                    self.condputs("\n");
                    return true;
                }
            }
            "[" => {
                self.refer = true;
                self.condputs("\n");
                return true;
            }
            "]" => {
                self.refer = false;
                self.skip_char(1);
                return self.text();
            }
            "." => {
                self.r#macro = false;
                self.condputs("\n");
                return true;
            }
            _ => (),
        };

        self.nobody = false;
        let s0s1 = self.s.clone().chars().take(2).collect::<String>();

        if self.g_macro_dispatch(s0s1.as_str()) {
            return true;
        }

        // TODO: This will never be true, SKIP_HEADERS is a const false
        if SKIP_HEADERS && self.nobody {
            return true;
        }

        self.skip_leading_whitespace();

        while !self.s.is_empty() && !self.is_white(0) {
            self.skip_char(1);
        }

        self.skip_leading_whitespace();

        loop {
            if !self.quoted_arg() && !self.text_arg() {
                if !self.s.is_empty() {
                    let s = &self.str_at(0).to_owned();
                    self.condputs(s);
                    self.skip_char(1);
                } else {
                    return true;
                }
            }
        }
    }

    fn font(&mut self) -> bool {
        if let Some(m) = self.g_re_font.find(&self.s.to_owned()) {
            self.skip_char(m.end());
            true
        } else {
            false
        }
    }

    fn comment(&mut self) -> bool {
        while !self.str_at(0).is_empty() && self.str_at(0) != "\n" {
            self.skip_char(1)
        }
        true
    }

    fn numreq(&mut self) -> bool {
        // In the python, it has a check that is already handled in esc_char_backslash, which is
        // the only place it gets called, so I'll omit that check here

        // This is written as `self.macro += 1` in the source, but I dont know why
        // it does the same thing (false -> true, true -> still true) :shrug:
        // self.r#macro = true;
        // Upon further investigation, this is the weirdest function ever
        // This is just a state placeholder thing
        let mut m = self.r#macro as u8 + 1;

        self.skip_char(3);

        // There's this, which has a comment explaining my thoughts right now
        /*
        while self.str_at(0) != "'" and self.esc_char():
            pass  # Weird
        */
        // And I dont know what the purpose of it would even be, if you can tell, lmk

        if self.str_at(0) == "\'" {
            self.skip_char(1);
        }

        m -= 1;
        self.r#macro = m != 0;

        true
    }

    // Rami wrote this originally, but I had to do a lot of work on it, soz
    fn text_arg(&mut self) -> bool {
        let mut got_something = false;
        loop {
            if self.s.is_empty() {
                self.condputs("\n");
                return got_something;
            } else if let Some("\\") = self.s.get(0..=0) {
                self.esc_char();
            } else if let Some(m) = self.g_re_not_backslash_or_whitespace.find(&self.s.clone()) {
                // Output the characters in the match
                // TODO: This is a shit fix for a bug where it would drop spaces in between certain elements of some macros. It would convert ".SH SEE ALSO" into "SEEALSO".
                let mut s = m.as_str().to_owned();
                self.skip_char(m.end());
                if self.s.starts_with(' ') {
                    s.push(' ');
                    self.skip_char(1);
                }
                self.condputs(&s);
                got_something = true;
            } else if !self.esc_char() {
                self.condputs(self.s.clone().get(0..=0).unwrap_or(""));
                self.skip_char(1);
                got_something = true;
            }
        }
    }

    fn prch(&self, idx: usize) -> bool {
        self.s
            .get(idx..=idx)
            .map(|c| !" \t\n".contains(c))
            .unwrap_or_default()
    }

    // This function is the worst, there are a few comments explaining some of it in the test (test_var)
    // its so hard to briefly put into words what this function does, basically depending on the state
    // of self.s, it will either, change self.s to "", a part of self.s, or a value in self.reg_table
    // which corresponds to a key that is part of self.s.
    // This should be like 2 or 3 functions, but it's only one. So there's that. :-)
    fn var(&mut self) -> bool {
        let s0s1 = &self.s[0..2];
        if s0s1 == "\\n" {
            if Some("dy") == self.s.get(3..5)
                || (self.str_at(2) == "(" && self.prch(3) && self.prch(4))
            {
                self.skip_char(5);
                return true;
            } else if self.str_at(2) == "[" && self.prch(3) {
                self.skip_char(3);
                while !self.str_at(0).is_empty() && self.str_at(0) != "]" {
                    self.skip_char(1);
                }
                return true;
            } else if self.prch(2) {
                self.skip_char(3);
                return true;
            } else {
                return false;
            }
        } else if s0s1 == "\\*" {
            let mut reg = String::new();
            if self.str_at(2) == "(" && self.prch(3) && self.prch(4) {
                reg = self.s[3..5].to_owned();
                self.skip_char(5);
            } else if self.str_at(2) == "[" && self.prch(3) {
                self.skip_char(3);
                while !self.str_at(0).is_empty() && self.str_at(0) != "]" {
                    reg.push_str(self.str_at(0));
                    self.skip_char(1);
                }
                if let Some("]") = self.s.get(0..1) {
                    self.skip_char(1);
                } else {
                    return false;
                }
            } else {
                return false;
            }

            if self.reg_table.contains_key(&reg) {
                // This unwrap is safe, i promis
                self.s = self.reg_table.get(&reg).unwrap().to_owned();
                self.text_arg();
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    }

    fn digit(&mut self, idx: usize) -> bool {
        self.str_at(idx).chars().all(|c| c.is_numeric())
    }

    fn size(&mut self) -> bool {
        /* # We require that the string starts with \s */
        if self.digit(2) || ("-+".contains(self.str_at(2)) && self.digit(3)) {
            self.skip_char(3);
            while self.digit(0) {
                self.skip_char(1);
            }
            true
        } else {
            false
        }
    }

    fn esc(&mut self) -> bool {
        /* # We require that the string start with backslash */
        // self.s.get(1..2).
        match self.s.to_owned().get(1..2) {
            Some(a) => match a {
                "e" | "E" => self.condputs("\\"),
                "t" => self.condputs("\t"),
                "0" | "~" => self.condputs(" "),
                "|" | "^" | "&" | ":" => (),
                o => self.condputs(o),
            },
            None => return false,
        };
        self.skip_char(2);
        true
    }

    fn word(&mut self) -> bool {
        let mut got_something = false;
        while let Some(m) = self.g_re_word.find(&self.s.clone()) {
            got_something = true;
            self.condputs(m.as_str());
            self.skip_char(m.end());

            while self.spec() {
                if !self.specletter {
                    break;
                }
            }
        }
        got_something
    }

    fn text(&mut self) -> bool {
        loop {
            if let Some(idx) = self.s.clone().find("\\") {
                self.condputs(self.s.clone().get(..idx).unwrap_or("")); // TODO: Fix! this may cause bugs later
                self.skip_char(idx);
                if !self.esc_char_backslash() {
                    self.condputs(self.s.clone().get(0..1).unwrap_or("")); // TODO: Fix! this may cause bugs later
                    self.skip_char(1);
                }
            } else {
                self.condputs(&self.s.clone());
                self.s = String::new();
                break;
            }
        }
        true
    }

    fn spec(&mut self) -> bool {
        self.specletter = false;
        if self.s.get(0..2).unwrap_or("") == "\\(" && self.prch(2) && self.prch(3) {
            let key = self.s.get(2..4).unwrap_or("");
            if let Some(k) = Deroffer::g_specs_specletter(key) {
                self.condputs(k);
                self.specletter = true;
            } else if let Some(k) = Deroffer::g_specs(key) {
                self.condputs(k);
            }
            self.skip_char(4);
            true
        } else if self.s.starts_with("\\%") {
            self.specletter = true;
            self.skip_char(2);
            true
        } else {
            false
        }
    }

    fn esc_char_backslash(&mut self) -> bool {
        if let Some(c) = self.s.get(1..2) {
            match c {
                "\"" => self.comment(),
                "f" => self.font(),
                "s" => self.size(),
                "h" | "v" | "w" | "u" | "d" => self.numreq(),
                "n" | "*" => self.var(),
                "(" => self.spec(),
                _ => self.esc(),
            }
        } else {
            false
        }
    }

    fn do_tbl(&mut self) -> bool {
        match self.tblstate {
            TblState::Options => {
                while !self.s.is_empty() && ";\n".contains(&self.s[0..=0]) {
                    self.skip_leading_whitespace();
                    if !self.str_at(0).chars().all(|c| c.is_alphabetic()) {
                        self.skip_char(1);
                    } else {
                        let mut option = self.s.clone();
                        let mut arg = String::new();

                        let mut idx = 0;
                        while option
                            .get(idx..=idx)
                            .unwrap_or("")
                            .chars()
                            .all(|c| c.is_alphabetic())
                        {
                            idx += 1;
                        }

                        if option.get(idx..=idx) == Some("(") {
                            option = option[..idx].to_owned();
                            self.s = self.s.get(idx + 1..).unwrap_or("").to_owned();
                            arg = self.s.clone();
                        } else {
                            self.s = String::new();
                        }

                        if !arg.is_empty() {
                            if arg.find(")") == None {
                                arg = arg[..idx].to_owned();
                            }
                            self.s = self.s[idx + 1..].to_owned();
                        } else {
                            // ?? self.skip_char(1);
                            // TODO: Investigate. deroff.py: 1083
                        }

                        if option.to_lowercase() == "tab" {
                            self.tblTab = arg[0..=0].to_owned();
                        }
                    }
                }

                self.tblstate = TblState::Format;
                self.condputs("\n");
            }
            TblState::Format => {
                while !self.s.is_empty() && ".\n".contains(&self.s[0..=0]) {
                    self.skip_leading_whitespace();
                    if !self.str_at(0).is_empty() {
                        self.skip_char(1);
                    }
                }

                if self.str_at(0) == "." {
                    self.tblstate = TblState::Data;
                }

                self.condputs("\n");
            }
            TblState::Data => {
                if !self.tblTab.is_empty() {
                    self.s = self.s.replace(&self.tblTab, "\t");
                }

                self.text();
            }
        }

        true
    }

    fn do_line(&mut self) -> bool {
        if ".'".contains(self.str_at(0)) {
            self.request_or_macro()
        } else if self.tbl {
            self.do_tbl()
        } else {
            self.text()
        }
    }

    fn deroff(&mut self, string: String) {
        for line in string.split("\n") {
            let mut line = line.to_owned();
            line.push('\n');
            self.s = line;
            if !self.do_line() {
                break;
            }
        }
    }

    fn flush_output<W: std::io::Write>(&mut self, mut write: W) {
        write!(write, "{}", self.output).expect("FAILED TO WRITE OUT");
    }
}

fn deroff_files(files: &[String]) -> std::io::Result<()> {
    for arg in files {
        let mut file = File::open(arg)?;
        let mut string = String::new();
        if arg.ends_with(".gz") {
            let mut decoder = Decoder::new(file).unwrap();
            decoder.read_to_string(&mut string);
        } else {
            file.read_to_string(&mut string)?;
        }
        let mut d = Deroffer::new();

        d.deroff(string);
        let mut output = std::fs::File::create("./example.deroff").unwrap();

        d.flush_output(output);
    }
    Ok(())
}

#[test]
fn test_get_output() {
    let deroffer = Deroffer::new();
    assert_eq!(&deroffer.get_output(b"foo\n\nbar").unwrap(), "foo\n\nbar");
    assert_eq!(&deroffer.get_output(b"foo\n\n\nbar").unwrap(), "foo\nbar");
}

#[test]
fn test_not_whitespace() {
    let mut d = Deroffer::new();

    assert_eq!(d.not_whitespace(0), false);
    assert_eq!(d.not_whitespace(9), false);
    d.s = "ab cd".to_owned();
    assert_eq!(d.not_whitespace(2), false);
    assert_eq!(d.not_whitespace(3), true);
}

#[test]
fn test_str_at() {
    let mut d = Deroffer::new();

    // d.s == ""
    assert_eq!(d.str_at(1), "");

    d.s = String::from("ab cd");
    assert_eq!(d.str_at(42), "");
    assert_eq!(d.str_at(1), "b");

    d.s = String::from("🗻");
    assert_eq!(d.str_at(0), "🗻");
    assert_eq!(d.str_at(1), "");
}

#[test]
fn test_is_white() {
    let mut d = Deroffer::new();
    assert_eq!(d.is_white(1), false);

    d.s = "ab cd".to_owned();
    assert_eq!(d.is_white(42), false); // OOB
    assert_eq!(d.is_white(1), false); // "b"
    assert_eq!(d.is_white(2), true); // " "
    assert_eq!(d.is_white(3), false); // "c"
}

#[test]
fn test_condputs() {
    let mut d = Deroffer::new();

    assert_eq!(d.output, String::new());
    d.condputs("Hello World!\n");
    assert_eq!(d.output, "Hello World!\n".to_owned());
    d.pic = true;
    d.condputs("This won't go to output");
    assert_eq!(d.output, "Hello World!\n".to_owned());
    d.pic = false;
    d.condputs("This will go to output :)");
    assert_eq!(
        d.output,
        "Hello World!\nThis will go to output :)".to_owned()
    );

    // Test the translation check
    d.tr = Some(maketrans("Ttr", "AAA"));
    d.condputs("Translate test");
    assert_eq!(
        d.output,
        "Hello World!\nThis will go to output :)AAanslaAe AesA".to_owned()
    );
}

#[test]
fn test_var() {
    let mut d = Deroffer::new();

    // "\n" successes
    d.s = "\\n dyHello".to_owned();
    assert!(d.var() == true);
    assert!(d.s == "Hello");

    d.s = "\\n(aaHello".to_owned();
    assert!(d.var() == true);
    assert!(d.s == "Hello");

    d.s = "\\n[skipme] Hello".to_owned();
    assert!(d.var() == true);
    assert!(d.s == "] Hello");

    d.s = "\\naHello".to_owned();
    assert!(d.var() == true);
    assert!(d.s == "Hello");

    // "\n" errors
    d.s = "\\n".to_owned();
    assert!(d.var() == false);
    assert!(d.s == "\\n");

    d.s = "\\n a".to_owned();
    assert!(d.var() == false);
    assert!(d.s == "\\n a");

    d.s = "\\n da".to_owned();
    assert!(d.var() == false);
    assert!(d.s == "\\n da");

    // "\*" successes

    // these blocks are more for me to understand the code,
    // but, I think they're probably helpful for you guys,
    // so here they are. On another note, I cannot WAIT to
    // reformat this entire project

    /*
    AA is two non whitespace characters
    "\*(AA" => {
        if AA in self.reg_table => {
            self.s = self.reg_table.get(AA);
            return true;
        } else => {
            self.s = self.s.get(5..);
            return false;
        }
    } */
    d.s = "\\*(traaaaaaaaaaaaa".to_owned();
    d.reg_table
        .insert("tr".to_owned(), "Hello World!".to_owned());
    assert!(d.var() == true);
    assert!(d.s == " World!");
    assert!(d.output.contains("Hello"));

    d.s = "\\*(aaHello World!".to_owned();
    assert!(d.var() == false);
    assert!(d.s == "Hello World!");

    /*
    A is a string that does not start with whitespace
    "\*[A" => {
        if A ends with "]" => {
            let B = A[..-1];

            if B in self.reg_table {
                self.s = self.reg_table.get(B);
                return true;
            } else {
                +1 to skip the "]" as well
                self.s = self.s[len(B)+1..];
                return false;
            }
        } else => {
            self.s = "";
            return false;
        }
    }
    */

    // ideal case, B is in reg_table
    d.s = "\\*[test_reg]".to_owned();
    d.reg_table
        .insert("test_reg".to_owned(), "It me!".to_owned());
    assert!(d.var() == true);
    assert!(d.s == " me!");
    assert!(d.output.contains("It"));

    // no "]"
    d.s = "\\*[foo bar :)".to_owned();
    assert!(d.var() == false);
    assert!(d.s == "");

    // B not in reg_table
    d.s = "\\*[foo bar]abcd".to_owned();
    assert!(d.var() == false);
    assert!(d.s == "abcd");

    // Here's a python version of these tests
    /*
        d.s = "\\n dyHello"
        print(d.var() == True)
        print(d.s == "Hello")

        d.s = "\\n(aaHello"
        print(d.var() == True)
        print(d.s == "Hello")

        d.s = "\\n[skipme] Hello"
        print(d.var() == True)
        print(d.s == "] Hello")

        d.s = "\\naHello"
        print(d.var() == True)
        print(d.s == "Hello")

        d.s = "\\n"
        print(d.var() == False)
        print(d.s == "\\n")

        d.s = "\\n a"
        print(d.var() == False)
        print(d.s == "\\n a")

        d.s = "\\n da"
        print(d.var() == False)
        print(d.s == "\\n da")

        d.s = "\\*(traaaaaaaaaaaaa"
        d.reg_table["tr"] = "Hello World!"
        print(d.var() == True)
        print(d.s == " World!")
        print("Hello" in d.output)

        d.s = "\\*(aaHello World!"
        print(d.var() == False)
        print(d.s == "Hello World!")

        d.s = "\\*[test_reg]"
        d.reg_table["test_reg"] = "It me!"
        print(d.var() == True)
        print(d.s == " me!")
        print("It" in d.output)

        d.s = "\\*[foo bar :)"
        print(d.var() == False)
        print(d.s == "")

        d.s = "\\*[foo bar]abcd"
        print(d.var() == False)
        print(d.s == "abcd")

        print(d.output)
    */
}

//     def __init__(self):
//         self.reg_table = {}
//         self.tr_from = ''
//         self.tr_to = ''
//         self.tr = ''
//         self.nls = 2
//         self.specletter = False
//         self.refer = False
//         self.macro = 0
//         self.nobody = False
//         self.inlist = False
//         self.inheader = False
//         self.pic = False
//         self.tbl = False
//         self.tblstate = 0
//         self.tblTab = ''
//         self.eqn = False
//         self.skipheaders = False
//         self.skiplists = False
//         self.ignore_sonx = False
//         self.output = []
//         self.name = ''
//
//         self.OPTIONS = 0
//         self.FORMAT = 1
//         self.DATA = 2
//
//         # words is uninteresting and should be treated as false
//
//     # This gets swapped in in place of condputs the first time tr gets modified
//     def condputs_tr(self, str):
//         special = self.pic or self.eqn or self.refer or self.macro or (self.skiplists and self.inlist) or (self.skipheaders and self.inheader)
//         if not special:
//             self.output.append(str.translate(self.tr))

//     def condputs(self, str):
//         special = self.pic or self.eqn or self.refer or self.macro or (self.skiplists and self.inlist) or (self.skipheaders and self.inheader)
//         if not special:
//             self.output.append(str)

//     def str_eq(offset, other, len):
//         return self.s[offset:offset+len] == other[:len]

//     def font(self):
//         match = Deroffer.g_re_font.match(self.s)
//         if not match: return False
//         self.skip_char(match.end())
//         return True

//     def comment(self):
//         # Here we require that the string start with \"
//         while self.str_at(0) and self.str_at(0) != '\n': self.skip_char()
//         return True

//     def numreq(self):
//         # We require that the string starts with backslash
//         if self.str_at(1) in 'hvwud' and self.str_at(2) == '\'':
//             self.macro += 1
//             self.skip_char(3)
//             while self.str_at(0) != '\'' and self.esc_char():
//                 pass # Weird
//             if self.str_at(0) == '\'':
//                 self.skip_char()
//             self.macro -= 1
//             return True
//         return False

//     def var(self):
//         reg = ''
//         s0s1 = self.s[0:2]
//         if s0s1 == '\\n':
//             if self.s[3:5] == 'dy':
//                 self.skip_char(5)
//                 return True
//             elif self.str_at(2) == '(' and self.not_whitespace(3) and self.not_whitespace(4):
//                 self.skip_char(5)
//                 return True
//             elif self.str_at(2) == '[' and self.not_whitespace(3):
//                 self.skip_char(3)
//                 while self.str_at(0) and self.str_at(0) != ']':
//                     self.skip_char()
//                 return True
//             elif self.not_whitespace(2):
//                 self.skip_char(3)
//                 return True
//         elif s0s1 == '\\*':
//             if self.str_at(2) == '(' and self.not_whitespace(3) and self.not_whitespace(4):
//                 reg = self.s[3:5]
//                 self.skip_char(5)
//             elif self.str_at(2) == '[' and self.not_whitespace(3):
//                 self.skip_char(3)
//                 while self.str_at(0) and self.str_at(0) != ']':
//                     reg = reg + self.str_at(0)
//                     self.skip_char()
//                 if self.s[0:1] == ']':
//                     self.skip_char()
//                 else:
//                     return False
//             elif self.not_whitespace(2):
//                 reg = self.str_at(2)
//                 self.skip_char(3)
//             else:
//                 return False
//
//             if reg in self.reg_table:
//                 old_s = self.s
//                 self.s = self.reg_table[reg]
//                 self.text_arg()
//                 return True
//         return False

//     def size(self):
//         # We require that the string starts with \s
//         if self.digit(2) or (self.str_at(2) in '-+' and self.digit(3)):
//             self.skip_char(3)
//             while self.digit(0): self.skip_char()
//             return True
//         return False

//     def spec(self):
//         self.specletter = False
//         if self.s[0:2] == '\\(' and self.not_whitespace(2) and self.not_whitespace(3):
//             key = self.s[2:4]
//             if key in Deroffer.g_specs_specletter:
//                 self.condputs(Deroffer.g_specs_specletter[key])
//                 self.specletter = True
//             elif key in Deroffer.g_specs:
//                 self.condputs(Deroffer.g_specs[key])
//             self.skip_char(4)
//             return True
//         elif self.s.startswith('\\%'):
//             self.specletter = True
//             self.skip_char(2)
//             return True
//         else:
//             return False

//     def esc(self):
//         # We require that the string start with backslash
//         c = self.s[1:2]
//         if not c: return False
//         if c in 'eE':
//             self.condputs('\\')
//         elif c in 't':
//             self.condputs('\t')
//         elif c in '0~':
//             self.condputs(' ')
//         elif c in '|^&:':
//             pass
//         else:
//             self.condputs(c)
//         self.skip_char(2)
//         return True

//     def word(self):
//         got_something = False
//         while True:
//             match = Deroffer.g_re_word.match(self.s)
//             if not match: break
//             got_something = True
//             self.condputs(match.group(0))
//             self.skip_char(match.end(0))
//
//             # Consume all specials
//             while self.spec():
//                 if not self.specletter: break
//
//         return got_something

//     def text(self):
//         while True:
//             idx = self.s.find('\\')
//             if idx == -1:
//                 self.condputs(self.s)
//                 self.s = ''
//                 break
//             else:
//                 self.condputs(self.s[:idx])
//                 self.skip_char(idx)
//                 if not self.esc_char_backslash():
//                     self.condputs(self.str_at(0))
//                     self.skip_char()
//         return True

//     def digit(self, idx):
//         ch = self.str_at(idx)
//         return ch.isdigit()

//     def number(self):
//         match = Deroffer.g_re_number.match(self.s)
//         if not match:
//             return False
//         else:
//             self.condputs(match.group(0))
//             self.skip_char(match.end())
//             return True

//     def esc_char_backslash(self):
//         # Like esc_char, but we know the string starts with a backslash
//         c = self.s[1:2]
//         if c == '"':
//             return self.comment()
//         elif c == 'f':
//             return self.font()
//         elif c == 's':
//             return self.size()
//         elif c in 'hvwud':
//             return self.numreq()
//         elif c in 'n*':
//             return self.var()
//         elif c == '(':
//             return self.spec()
//         else:
//             return self.esc()

//     def esc_char(self):
//         if self.s[0:1] == '\\':
//             return self.esc_char_backslash()
//         return self.word() or self.number()

//     def quoted_arg(self):
//         if self.str_at(0) == '"':
//             self.skip_char()
//             while self.s and self.str_at(0) != '"':
//                 if not self.esc_char():
//                     if self.s:
//                         self.condputs(self.str_at(0))
//                         self.skip_char()
//             return True
//         else:
//             return False

//     def text_arg(self):
//         # PCA: The deroff.c textArg() disallowed quotes at the start of an argument
//         # I'm not sure if this was a bug or not
//         got_something = False
//         while True:
//             match = Deroffer.g_re_not_backslash_or_whitespace.match(self.s)
//             if match:
//                 # Output the characters in the match
//                 self.condputs(match.group(0))
//                 self.skip_char(match.end(0))
//                 got_something = True
//
//             # Next is either an escape, or whitespace, or the end
//             # If it's the whitespace or the end, we're done
//             if not self.s or self.is_white(0):
//                 return got_something
//
//             # Try an escape
//             if not self.esc_char():
//                 # Some busted escape? Just output it
//                 self.condputs(self.str_at(0))
//                 self.skip_char()
//                 got_something = True

//     def text_arg2(self):
//         if not self.esc_char():
//             if self.s and not self.is_white(0):
//                 self.condputs(self.str_at(0))
//                 self.skip_char()
//             else:
//                 return False
//         while True:
//             if not self.esc_char():
//                 if self.s and not self.is_white(0):
//                     self.condputs(self.str_at(0))
//                     self.skip_char()
//                 else:
//                     return True

//     # Macro functions
//     def macro_sh(self):
//         for header_str in [' SYNOPSIS', ' "SYNOPSIS', ' ‹BERSICHT', ' "‹BERSICHT']:
//             if self.s[2:].startswith(header_str):
//                 self.inheader = True
//                 break
//         else:
//             # Did not find a header string
//             self.inheader = False
//             self.nobody = True

//     def macro_ss_ip(self):
//         self.nobody = True
//         return False

//     def macro_i_ir(self):
//         pass
//         return False

//     def macro_nm(self):
//         if self.s == 'Nm\n':
//             self.condputs(self.name)
//         else:
//             self.name = self.s[3:].strip() + ' '
//         return True

//     def macro_close_bracket(self):
//         self.refer = False
//         return False

//     def macro_ps(self):
//         if self.is_white(2): self.pic = True
//         self.condputs('\n')
//         return True

//     def macro_pe(self):
//         if self.is_white(2): self.pic = False
//         self.condputs('\n')
//         return True

//     def macro_ts(self):
//         if self.is_white(2): self.tbl, self.tblstate = True, self.OPTIONS
//         self.condputs('\n')
//         return True

//     def macro_t_and(self):
//         if self.is_white(2): self.tbl, self.tblstate = True, self.FORMAT
//         self.condputs('\n')
//         return True

//     def macro_te(self):
//         if self.is_white(2): self.tbl = False
//         self.condputs('\n')
//         return True

//     def macro_eq(self):
//         if self.is_white(2): self.eqn = True
//         self.condputs('\n')
//         return True

//     def macro_en(self):
//         if self.is_white(2): self.eqn = False
//         self.condputs('\n')
//         return True

//     def macro_r1(self):
//         if self.is_white(2): self.refer2 = True
//         self.condputs('\n')
//         return True

//     def macro_r2(self):
//         if self.is_white(2): self.refer2 = False
//         self.condputs('\n')
//         return True

//     def macro_de(self):
//         macro=True
//         self.condputs('\n')
//         return True

//     def macro_bl_vl(self):
//         if self.is_white(2): self.inlist = True
//         self.condputs('\n')
//         return True

//     def macro_bv(self):
//         if self.str_at(2) == 'L' and self.white(self.str_at(3)): self.inlist = True
//         self.condputs('\n')
//         return True

//     def macro_le(self):
//         if self.is_white(2): self.inlist = False
//         self.condputs('\n')
//         return True

//     def macro_lp_pp(self):
//         self.condputs('\n')
//         return True

//     def macro_ds(self):
//         self.skip_char(2)
//         self.skip_leading_whitespace()
//         if self.str_at(0):
//             # Split at whitespace
//             comps = self.s.split(None, 2)
//             if len(comps) is 2:
//                 name, value = comps
//                 value = value.rstrip()
//                 self.reg_table[name] = value
//         self.condputs('\n')
//         return True

//     def macro_so_nx(self):
//         # We always ignore include directives
//         # deroff.c for some reason allowed this to fall through to the 'tr' case
//         # I think that was just a bug so I won't replicate it
//         return True

//     def macro_tr(self):
//         self.skip_char(2)
//         self.skip_leading_whitespace()
//         while self.s and self.str_at(0) != '\n':
//             c = self.str_at(0)
//             ns = self.str_at(1)
//             self.skip_char(2)
//             if not ns or ns == '\n': ns = ' '
//             self.tr_from += c
//             self.tr_to += ns
//
//         # Update our table, then swap in the slower tr-savvy condputs
//         try: #Python2
//             self.tr = string.maketrans(self.tr_from, self.tr_to)
//         except AttributeError: #Python3
//             self.tr = "".maketrans(self.tr_from, self.tr_to)
//         self.condputs = self.condputs_tr
//         return True

//     def macro_sp(self):
//         self.condputs('\n')
//         return True

//     def macro_other(self):
//         self.condputs('\n')
//         return True

//     def request_or_macro(self):
//         # s[0] is period or open single quote
//         self.skip_char()
//         s0 = self.s[1:2]
//         if s0 == '\\':
//             if self.str_at(1) == '"':
//                 self.condputs('\n')
//                 return True
//             else:
//                 pass
//         elif s0 == '[':
//             self.refer = True
//             self.condputs('\n')
//             return True
//         elif s0 == ']':
//             self.refer = False
//             self.skip_char()
//             return self.text()
//         elif s0 == '.':
//             self.macro = False
//             self.condputs('\n')
//             return True
//
//         self.nobody = False
//         s0s1 = self.s[0:2]
//
// RUST NOTE: use Deroffer.g_macro_dispatch(s0s1) which will return like macro_func does below
//         macro_func = Deroffer.g_macro_dict.get(s0s1, Deroffer.macro_other)
//         if macro_func(self):
//             return True
//
//         if self.skipheaders and self.nobody: return True
//
//         self.skip_leading_whitespace()
//         while self.s and not self.is_white(0): self.skip_char()
//         self.skip_leading_whitespace()
//         while True:
//             if not self.quoted_arg() and not self.text_arg():
//                 if self.s:
//                     self.condputs(self.str_at(0))
//                     self.skip_char()
//                 else:
//                     return True

//     def request_or_macro2(self):
//         self.skip_char()
//         s0 = self.s[0:1]
//         if s0 == '\\':
//             if self.str_at(1) == '"':
//                 self.condputs('\n')
//                 return True
//             else:
//                 pass
//         elif s0 == '[':
//             self.refer = True
//             self.condputs('\n')
//             return True
//         elif s0 == ']':
//             self.refer = False
//             self.skip_char()
//             return self.text()
//         elif s0 == '.':
//             self.macro = False
//             self.condputs('\n')
//             return True
//
//         self.nobody = False
//         s0s1 = self.s[0:2]
//         if s0s1 == 'SH':
//             for header_str in [' SYNOPSIS', ' "SYNOPSIS', ' ‹BERSICHT', ' "‹BERSICHT']:
//                 if self.s[2:].startswith(header_str):
//                     self.inheader = True
//                     break
//             else:
//                 # Did not find a header string
//                 self.inheader = False
//                 self.nobody = True
//         elif s0s1 in ['SS', 'IP', 'H ']:
//             self.nobody = True
//         elif s0s1 in ['I ', 'IR', 'IB', 'B ', 'BR', 'BI', 'R ', 'RB', 'RI', 'AB']:
//             pass
//         elif s0s1 in ['] ']:
//             self.refer = False
//         elif s0s1 in ['PS']:
//             if self.is_white(2): self.pic = True
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['PE']:
//             if self.is_white(2): self.pic = False
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['TS']:
//             if self.is_white(2): self.tbl, self.tblstate = True, self.OPTIONS
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['T&']:
//             if self.is_white(2): self.tbl, self.tblstate = True, self.FORMAT
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['TE']:
//             if self.is_white(2): self.tbl = False
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['EQ']:
//             if self.is_white(2): self.eqn = True
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['EN']:
//             if self.is_white(2): self.eqn = False
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['R1']:
//             if self.is_white(2): self.refer2 = True
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['R2']:
//             if self.is_white(2): self.refer2 = False
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['de']:
//             macro=True
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['BL', 'VL', 'AL', 'LB', 'RL', 'ML', 'DL']:
//             if self.is_white(2): self.inlist = True
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['BV']:
//             if self.str_at(2) == 'L' and self.white(self.str_at(3)): self.inlist = True
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['LE']:
//             if self.is_white(2): self.inlist = False
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['LP', 'PP', 'P\n']:
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['ds']:
//             self.skip_char(2)
//             self.skip_leading_whitespace()
//             if self.str_at(0):
//                 # Split at whitespace
//                 comps = self.s.split(None, 2)
//                 if len(comps) is 2:
//                     name, value = comps
//                     value = value.rstrip()
//                     self.reg_table[name] = value
//             self.condputs('\n')
//             return True
//         elif s0s1 in ['so', 'nx']:
//             # We always ignore include directives
//             # deroff.c for some reason allowed this to fall through to the 'tr' case
//             # I think that was just a bug so I won't replicate it
//             return True
//         elif s0s1 in ['tr']:
//             self.skip_char(2)
//             self.skip_leading_whitespace()
//             while self.s and self.str_at(0) != '\n':
//                 c = self.str_at(0)
//                 ns = self.str_at(1)
//                 self.skip_char(2)
//                 if not ns or ns == '\n': ns = ' '
//                 self.tr_from += c
//                 self.tr_to += ns
//
//             # Update our table, then swap in the slower tr-savvy condputs
//             try: #Python2
//                 self.tr = string.maketrans(self.tr_from, self.tr_to)
//             except AttributeError: #Python3
//                 self.tr = "".maketrans(self.tr_from, self.tr_to)
//             self.condputs = self.condputs_tr
//
//             return True
//         elif s0s1 in ['sp']:
//             self.condputs('\n')
//             return True
//         else:
//             self.condputs('\n')
//             return True
//
//         if self.skipheaders and self.nobody: return True
//
//         self.skip_leading_whitespace()
//         while self.s and not self.is_white(0): self.skip_char()
//         self.skip_leading_whitespace()
//         while True:
//             if not self.quoted_arg() and not self.text_arg():
//                 if self.s:
//                     self.condputs(self.str_at(0))
//                     self.skip_char()
//                 else:
//                     return True

//     def do_tbl(self):
//         if self.tblstate == self.OPTIONS:
//             while self.s and self.str_at(0) != ';' and self.str_at(0) != '\n':
//                 self.skip_leading_whitespace()
//                 if not self.str_at(0).isalpha():
//                     # deroff.c has a bug where it can loop forever here...we try to work around it
//                     self.skip_char()
//                 else: # Parse option
//
//                     option = self.s
//                     arg = ''
//
//                     idx = 0
//                     while option[idx:idx+1].isalpha():
//                         idx += 1
//
//                     if option[idx:idx+1] == '(':
//                         option = option[:idx]
//                         self.s = self.s[idx+1:]
//                         arg = self.s
//                     else:
//                         self.s = ''
//
//                     if arg:
//                         idx = arg.find(')')
//                         if idx != -1:
//                             arg = arg[:idx]
//                         self.s = self.s[idx+1:]
//                     else:
//                         #self.skip_char()
//                         pass
//
//                     if option.lower() == 'tab':
//                         self.tblTab = arg[0:1]
//
//             self.tblstate = self.FORMAT
//             self.condputs('\n')
//
//         elif self.tblstate == self.FORMAT:
//             while self.s and self.str_at(0) != '.' and self.str_at(0) != '\n':
//                 self.skip_leading_whitespace()
//                 if self.str_at(0): self.skip_char()
//
//             if self.str_at(0) == '.': self.tblstate = self.DATA
//             self.condputs('\n')
//         elif self.tblstate == self.DATA:
//             if self.tblTab:
//                 self.s = self.s.replace(self.tblTab, '\t')
//             self.text()
//         return True

//     def do_line(self):
//         if self.s[0:1] in ".'":
//             if not self.request_or_macro(): return False
//         elif self.tbl:
//             self.do_tbl()
//         else:
//             self.text()
//         return True

//     def deroff(self, str):
//         lines = str.split('\n')
//         for line in lines:
//             self.s = line + '\n'
//             if not self.do_line():
//                 break

// if __name__ == "__main__":
//     import gzip
//     paths = sys.argv[1:]
//     if True:
//         deroff_files(paths)
//     else:
//         import cProfile, profile, pstats
//         profile.run('deroff_files(paths)', 'fooprof')
//         p = pstats.Stats('fooprof')
//         p.sort_stats('time').print_stats(100)
//         #p.sort_stats('calls').print_callers(.5, 'startswith')
