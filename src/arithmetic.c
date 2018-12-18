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

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>

#include "bindings.h"
#include "arithmetic.h"

int execIExpression(char **str, unsigned char *bInPtr, char bitpos, char *pPtr, char *err)
{
    int f = 1;
    int term1, term2;
    //int exp1, exp2;
    int op;
    char *item;
    unsigned char bPtr[10];
    int n;

    //printf("execExpression: %s\n", *str);

    // Tweak bPtr bytes 0..9 and copy them to nPtr
    // We have received characters
    for (n = 0; n <= 9; n++) {
        //bPtr[n]=*bInPtr++ & 255;
        bPtr[n] = *bInPtr++;
    }

    op = ERROR;
    switch (nextToken(str, &item, &n)) {
    case PLUS:
        op = PLUS;
        break;
    case MINUS:
        op = MINUS;
        break;
    case NICHT:
        op = NICHT;
        break;
    default:
        pushBack(str, n);
        break;
    }

    if (op == MINUS) {
        term1 = execITerm(str, bPtr, bitpos, pPtr, err) * -1;
    } else if (op == NICHT) {
        term1 = ~(execITerm(str, bPtr, bitpos, pPtr, err));
    } else {
        term1 = execITerm(str, bPtr, bitpos, pPtr, err);
    }

    if (*err) {
        return 0;
    }

    int t;
    op = ERROR;
    while ((t = nextToken(str, &item, &n)) != END) {
        f = 1;
        switch (t) {
        case PLUS:
            op = PLUS;
            break;
        case MINUS:
            op = MINUS;
            break;
        case NICHT:
            op = NICHT;
            break;
        default:
            pushBack(str, n);
            return term1;
        }

        if (op == MINUS) {
            term2 = execITerm(str, bPtr, bitpos, pPtr, err) * -1;
        } else if (op == NICHT) {
            term2 = ~(execITerm(str, bPtr, bitpos, pPtr, err));
        } else if (op == PLUS) {
            term2 = execITerm(str, bPtr, bitpos, pPtr, err);
        } if (*err) {
            return 0;
        }
        term1 += term2;
    }

    return term1;
}

void  pushBack(char **str, int count)
{
    (*str) -= count;
    //printf("\t<<::%s\n",*str);
}
