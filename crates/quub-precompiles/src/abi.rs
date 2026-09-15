//! ABI selectors for F201–F203. Alloy 0.8 `sol!` — no reth types.

use alloy_sol_types::sol;

sol! {
    function check(
        address token,
        address from,
        address to,
        uint256 amount,
        bytes32 trHash
    ) external view returns (bool allowed, uint16 reason);

    function validateAndCommit(
        bytes32 endToEndId,
        bytes16 uetr,
        bytes32 instrId,
        bytes3 ccy,
        uint8 msgType
    ) external view returns (bytes32 memoHash);

    function quote(address feeToken, uint256 gasLimit, uint256 gasPrice)
        external
        view
        returns (uint256 tokenAmount);

    function takeFee(address payer, address feeToken, uint256 tokenAmount) external;
}

pub use checkCall as CheckCall;
pub use quoteCall as QuoteCall;
pub use takeFeeCall as TakeFeeCall;
pub use validateAndCommitCall as ValidateAndCommitCall;
