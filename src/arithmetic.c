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

float execExpression(char **str, unsigned char *bInPtr, float floatV, char *err)
{
    int f = 1;
    float term1, term2;
    //float exp1,exp2;
    char *item;
    //unsigned char bPtr[10];
    unsigned char bPtr[10];
    int n;

    //printf("execExpression: %s\n",*str);

    // Tweak bPtr Bytes 0..9 and copy them to nPtr
    // We did not receive characters
    for (n = 0; n <= 9; n++) {
        //bPtr[n]=*bInPtr++ & 255;
        bPtr[n] = *bInPtr++;
    }

    switch (nextToken(str, &item, &n)) {
    case PLUS:
        f = 1;
        break;
    case MINUS:
        f = -1;
        break;
    default:
        pushBack(str, n);
        break;
    }

    term1 = execTerm(str, bPtr, floatV, err) * f;
    if (*err) {
        return 0;
    }
    //printf(" T1=%f\n",term1);

    int t;
    while ((t = nextToken(str, &item, &n)) != END) {
        f = 1;
        switch (t) {
        case PLUS:
            f = 1;
            break;
        case MINUS:
            f = -1;
            break;
        default:
            //printf(" Exp=%f\n",term1);
            pushBack(str, n);
            return term1;
        }

        term2 = execTerm(str, bPtr, floatV, err);
        if (*err) {
            return 0;
        }
        //printf(" T2=%f\n",term2);
        term1 += term2 * f;
    }

    //printf(" Exp=%f\n",term1);
    return term1;
}

float execTerm(char **str, unsigned char *bPtr, float floatV, char *err)
{
    float factor1, factor2;
    int op;
    char *item;
    int n;

    //printf("execTerm: %s\n",*str);

    factor1 = execFactor(str, bPtr, floatV, err);
    if (*err) {
        return 0;
    }

    //printf(" F1=%f\n",factor1);
    while (1) {
        switch (nextToken(str, &item, &n)) {
        case MAL:
            op = MAL;
            break;
        case GETEILT:
            op = GETEILT;
            break;
        default:
            pushBack(str, n);
            //printf("  ret(%f)\n",factor1);
            return factor1;
        }
        factor2 = execFactor(str, bPtr, floatV, err);
        //printf(" F2=%f\n",factor2);
        if (*err) {
            return 0;
        }
        if (op == MAL) {
            factor1 *= factor2;
        } else {
            factor1 /= factor2;
        }
    }
}

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

int execITerm(char **str, unsigned char *bPtr, char bitpos, char *pPtr, char *err)
{
    int factor1, factor2;
    int op;
    char *item;
    int n;

    //printf("execTerm: %s\n",*str);

    factor1 = execIFactor(str, bPtr, bitpos, pPtr, err);
    if (*err) {
        return 0;
    }

    while (1) {
        switch (nextToken(str, &item, &n)) {
        case MAL:
            op = MAL;
            break;
        case GETEILT:
            op = GETEILT;
            break;
        case MODULO:
            op = MODULO;
            break;
        case UND:
            op = UND;
            break;
        case ODER:
            op = ODER;
            break;
        case XOR:
            op = XOR;
            break;
        case SHL:
            op = SHL;
            break;
        case SHR:
            op = SHR;
            break;
        default:
            pushBack(str, n);
            //printf("  ret(%f)\n",factor1);
            return factor1;
        }

        factor2 = execIFactor(str, bPtr, bitpos, pPtr, err);

        if (*err) {
            return 0;
        }

        if (op == MAL) {
            factor1 *= factor2;
        } else if (op == GETEILT) {
            factor1 /= factor2;
        } else if (op == MODULO) {
            factor1 %= factor2;
        } else if (op == UND) {
            factor1 &= factor2;
        } else if (op == ODER) {
            factor1 |= factor2;
        } else if (op == XOR) {
            factor1 ^= factor2;
        } else if (op == SHL) {
            factor1 <<= factor2;
        } else if (op == SHR) {
            factor1 >>= factor2;
        } else {
            sprintf(err, "Error exec ITerm: Unknown token %d", op);
            return 0;
        }
    }
}

void  pushBack(char **str, int count)
{
    (*str) -= count;
    //printf("\t<<::%s\n",*str);
}
