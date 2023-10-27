import typing
from anchorpy.error import ProgramError


class InsufficientUnlockedTokens(ProgramError):
    def __init__(self) -> None:
        super().__init__(6000, "Insufficient unlocked tokens")

    code = 6000
    name = "InsufficientUnlockedTokens"
    msg = "Insufficient unlocked tokens"


class DepositStartTooFarInFuture(ProgramError):
    def __init__(self) -> None:
        super().__init__(6001, "Deposit Start too far in future")

    code = 6001
    name = "DepositStartTooFarInFuture"
    msg = "Deposit Start too far in future"


class InvalidProof(ProgramError):
    def __init__(self) -> None:
        super().__init__(6002, "Invalid Merkle proof.")

    code = 6002
    name = "InvalidProof"
    msg = "Invalid Merkle proof."


class ExceededMaxClaim(ProgramError):
    def __init__(self) -> None:
        super().__init__(6003, "Exceeded maximum claim amount.")

    code = 6003
    name = "ExceededMaxClaim"
    msg = "Exceeded maximum claim amount."


class ExceededMaxNumNodes(ProgramError):
    def __init__(self) -> None:
        super().__init__(6004, "Exceeded maximum number of claimed nodes.")

    code = 6004
    name = "ExceededMaxNumNodes"
    msg = "Exceeded maximum number of claimed nodes."


class Unauthorized(ProgramError):
    def __init__(self) -> None:
        super().__init__(6005, "Account is not authorized to execute this instruction")

    code = 6005
    name = "Unauthorized"
    msg = "Account is not authorized to execute this instruction"


class OwnerMismatch(ProgramError):
    def __init__(self) -> None:
        super().__init__(6006, "Token account owner did not match intended owner")

    code = 6006
    name = "OwnerMismatch"
    msg = "Token account owner did not match intended owner"


class ClawbackBeforeVestingEnd(ProgramError):
    def __init__(self) -> None:
        super().__init__(6007, "Clawback cannot be before vesting ends")

    code = 6007
    name = "ClawbackBeforeVestingEnd"
    msg = "Clawback cannot be before vesting ends"


class ClawbackBeforeStart(ProgramError):
    def __init__(self) -> None:
        super().__init__(6008, "Attempting to clawback before clawback start")

    code = 6008
    name = "ClawbackBeforeStart"
    msg = "Attempting to clawback before clawback start"


class ClawbackAlreadyClaimed(ProgramError):
    def __init__(self) -> None:
        super().__init__(6009, "Clawback already claimed")

    code = 6009
    name = "ClawbackAlreadyClaimed"
    msg = "Clawback already claimed"


class ClawbackNewReceiverCannotBeSame(ProgramError):
    def __init__(self) -> None:
        super().__init__(6010, "New Clawback Receiver cannot be same as old")

    code = 6010
    name = "ClawbackNewReceiverCannotBeSame"
    msg = "New Clawback Receiver cannot be same as old"


class NewAdminCannotBeSame(ProgramError):
    def __init__(self) -> None:
        super().__init__(6011, "New Admin cannot be same as old")

    code = 6011
    name = "NewAdminCannotBeSame"
    msg = "New Admin cannot be same as old"


class ClaimExpired(ProgramError):
    def __init__(self) -> None:
        super().__init__(6012, "Cannot create claim; claim window expired")

    code = 6012
    name = "ClaimExpired"
    msg = "Cannot create claim; claim window expired"


class ArithmeticError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6013, "Arithmetic Error (overflow/underflow)")

    code = 6013
    name = "ArithmeticError"
    msg = "Arithmetic Error (overflow/underflow)"


class InvalidTimestamp(ProgramError):
    def __init__(self) -> None:
        super().__init__(6014, "Invalid Timestamp")

    code = 6014
    name = "InvalidTimestamp"
    msg = "Invalid Timestamp"


class InvalidVersion(ProgramError):
    def __init__(self) -> None:
        super().__init__(6015, "Airdrop Version Mismatch")

    code = 6015
    name = "InvalidVersion"
    msg = "Airdrop Version Mismatch"


CustomError = typing.Union[
    InsufficientUnlockedTokens,
    DepositStartTooFarInFuture,
    InvalidProof,
    ExceededMaxClaim,
    ExceededMaxNumNodes,
    Unauthorized,
    OwnerMismatch,
    ClawbackBeforeVestingEnd,
    ClawbackBeforeStart,
    ClawbackAlreadyClaimed,
    ClawbackNewReceiverCannotBeSame,
    NewAdminCannotBeSame,
    ClaimExpired,
    ArithmeticError,
    InvalidTimestamp,
    InvalidVersion,
]
CUSTOM_ERROR_MAP: dict[int, CustomError] = {
    6000: InsufficientUnlockedTokens(),
    6001: DepositStartTooFarInFuture(),
    6002: InvalidProof(),
    6003: ExceededMaxClaim(),
    6004: ExceededMaxNumNodes(),
    6005: Unauthorized(),
    6006: OwnerMismatch(),
    6007: ClawbackBeforeVestingEnd(),
    6008: ClawbackBeforeStart(),
    6009: ClawbackAlreadyClaimed(),
    6010: ClawbackNewReceiverCannotBeSame(),
    6011: NewAdminCannotBeSame(),
    6012: ClaimExpired(),
    6013: ArithmeticError(),
    6014: InvalidTimestamp(),
    6015: InvalidVersion(),
}


def from_code(code: int) -> typing.Optional[CustomError]:
    maybe_err = CUSTOM_ERROR_MAP.get(code)
    if maybe_err is None:
        return None
    return maybe_err
