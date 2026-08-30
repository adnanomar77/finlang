(* FinLang proof artifact. This file is intentionally compatible with Coq's Gallina syntax. *)
From Coq Require Import Arith Lia.
Inductive Value := VNat (n:nat) | VBool (b:bool).
Inductive Op := Add | Sub | Mul | Div | Lt | Gt | Le | Ge | Eq.
Definition nat_bin (op:Op) (a b:nat) : option Value :=
  match op with
  | Add => Some (VNat (a+b)) | Sub => if b <=? a then Some (VNat (a-b)) else None
  | Mul => Some (VNat (a*b)) | Div => if b =? 0 then None else Some (VNat (a/b))
  | Lt => Some (VBool (a <? b)) | Gt => Some (VBool (b <? a))
  | Le => Some (VBool (a <=? b)) | Ge => Some (VBool (b <=? a)) | Eq => Some (VBool (a =? b))
  end.
Theorem subtraction_safe : forall a b n, nat_bin Sub a b = Some (VNat n) -> b <= a.
Proof. intros a b n H. unfold nat_bin in H. destruct (b <=? a) eqn:E; inversion H; apply Nat.leb_le; exact E. Qed.
Theorem division_by_zero_rejected : forall a n, nat_bin Div a 0 <> Some (VNat n).
Proof. intros a n H. simpl in H. discriminate. Qed.
(* Progress/preservation for the full financial language remain parameterized by the
   state and typing judgments and must be completed against a formalized semantics. *)
