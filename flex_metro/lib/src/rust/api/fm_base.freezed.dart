// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'fm_base.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$MetricKey {
  (Object, int) get field0;

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MetricKey &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'MetricKey(field0: $field0)';
  }
}

/// @nodoc
class $MetricKeyCopyWith<$Res> {
  $MetricKeyCopyWith(MetricKey _, $Res Function(MetricKey) __);
}

/// Adds pattern-matching-related methods to [MetricKey].
extension MetricKeyPatterns on MetricKey {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(MetricKey_Standard value)? standard,
    TResult Function(MetricKey_Composite value)? composite,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case MetricKey_Standard() when standard != null:
        return standard(_that);
      case MetricKey_Composite() when composite != null:
        return composite(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(MetricKey_Standard value) standard,
    required TResult Function(MetricKey_Composite value) composite,
  }) {
    final _that = this;
    switch (_that) {
      case MetricKey_Standard():
        return standard(_that);
      case MetricKey_Composite():
        return composite(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(MetricKey_Standard value)? standard,
    TResult? Function(MetricKey_Composite value)? composite,
  }) {
    final _that = this;
    switch (_that) {
      case MetricKey_Standard() when standard != null:
        return standard(_that);
      case MetricKey_Composite() when composite != null:
        return composite(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function((int, int) field0)? standard,
    TResult Function((Int32List, int) field0)? composite,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case MetricKey_Standard() when standard != null:
        return standard(_that.field0);
      case MetricKey_Composite() when composite != null:
        return composite(_that.field0);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function((int, int) field0) standard,
    required TResult Function((Int32List, int) field0) composite,
  }) {
    final _that = this;
    switch (_that) {
      case MetricKey_Standard():
        return standard(_that.field0);
      case MetricKey_Composite():
        return composite(_that.field0);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function((int, int) field0)? standard,
    TResult? Function((Int32List, int) field0)? composite,
  }) {
    final _that = this;
    switch (_that) {
      case MetricKey_Standard() when standard != null:
        return standard(_that.field0);
      case MetricKey_Composite() when composite != null:
        return composite(_that.field0);
      case _:
        return null;
    }
  }
}

/// @nodoc

class MetricKey_Standard extends MetricKey {
  const MetricKey_Standard(this.field0) : super._();

  @override
  final (int, int) field0;

  /// Create a copy of MetricKey
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MetricKey_StandardCopyWith<MetricKey_Standard> get copyWith =>
      _$MetricKey_StandardCopyWithImpl<MetricKey_Standard>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MetricKey_Standard &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'MetricKey.standard(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $MetricKey_StandardCopyWith<$Res>
    implements $MetricKeyCopyWith<$Res> {
  factory $MetricKey_StandardCopyWith(
          MetricKey_Standard value, $Res Function(MetricKey_Standard) _then) =
      _$MetricKey_StandardCopyWithImpl;
  @useResult
  $Res call({(int, int) field0});
}

/// @nodoc
class _$MetricKey_StandardCopyWithImpl<$Res>
    implements $MetricKey_StandardCopyWith<$Res> {
  _$MetricKey_StandardCopyWithImpl(this._self, this._then);

  final MetricKey_Standard _self;
  final $Res Function(MetricKey_Standard) _then;

  /// Create a copy of MetricKey
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(MetricKey_Standard(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as (int, int),
    ));
  }
}

/// @nodoc

class MetricKey_Composite extends MetricKey {
  const MetricKey_Composite(this.field0) : super._();

  @override
  final (Int32List, int) field0;

  /// Create a copy of MetricKey
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MetricKey_CompositeCopyWith<MetricKey_Composite> get copyWith =>
      _$MetricKey_CompositeCopyWithImpl<MetricKey_Composite>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MetricKey_Composite &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'MetricKey.composite(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $MetricKey_CompositeCopyWith<$Res>
    implements $MetricKeyCopyWith<$Res> {
  factory $MetricKey_CompositeCopyWith(
          MetricKey_Composite value, $Res Function(MetricKey_Composite) _then) =
      _$MetricKey_CompositeCopyWithImpl;
  @useResult
  $Res call({(Int32List, int) field0});
}

/// @nodoc
class _$MetricKey_CompositeCopyWithImpl<$Res>
    implements $MetricKey_CompositeCopyWith<$Res> {
  _$MetricKey_CompositeCopyWithImpl(this._self, this._then);

  final MetricKey_Composite _self;
  final $Res Function(MetricKey_Composite) _then;

  /// Create a copy of MetricKey
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(MetricKey_Composite(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as (Int32List, int),
    ));
  }
}

// dart format on
