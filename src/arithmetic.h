/*  Copyright 2007-2017 the original vcontrold development team

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// Calculation of arithmetic expressions

#ifndef ARITHMETIC_H
#define ARITHMETIC_H

static const int HEX = 8;
static const int HEXDIGIT = 10;
static const int DIGIT = 11;
static const int PUNKT = 12;
static const int END = 0;
static const int ERROR = -100;
static const int PLUS = 100;
static const int MINUS = 101;
static const int MAL = 102;
static const int GETEILT = 103;
static const int MODULO = 104;
static const int KAUF = 110;
static const int KZU = 111;
static const int BYTE0 = 200;
static const int BYTE1 = 201;
static const int BYTE2 = 202;
static const int BYTE3 = 203;
static const int BYTE4 = 204;
static const int BYTE5 = 205;
static const int BYTE6 = 206;
static const int BYTE7 = 207;
static const int BYTE8 = 208;
static const int BYTE9 = 209;
static const int PBYTE0 = 210;
static const int PBYTE1 = 211;
static const int PBYTE2 = 212;
static const int PBYTE3 = 213;
static const int PBYTE4 = 214;
static const int PBYTE5 = 215;
static const int PBYTE6 = 216;
static const int PBYTE7 = 217;
static const int PBYTE8 = 218;
static const int PBYTE9 = 219;
static const int BITPOS = 220;
static const int VALUE = 300;
static const int NICHT = 400;
static const int UND = 401;
static const int ODER = 402;
static const int XOR = 403;
static const int SHL = 404;
static const int SHR = 405;

void  pushBack(char **str, int n);

#endif // ARITHMETIC_H
