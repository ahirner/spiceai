select
    s_name,
    count(*) as numwait
from
    tpch.tpch.supplier,
    tpch.tpch.lineitem l1,
    tpch.tpch.orders,
    tpch.tpch.nation
where
        s_suppkey = l1.l_suppkey
  and o_orderkey = l1.l_orderkey
  and o_orderstatus = 'F'
  and l1.l_receiptdate > l1.l_commitdate
  and exists (
        select
            *
        from
            tpch.tpch.lineitem l2
        where
                l2.l_orderkey = l1.l_orderkey
          and l2.l_suppkey <> l1.l_suppkey
    )
  and not exists (
        select
            *
        from
            tpch.tpch.lineitem l3
        where
                l3.l_orderkey = l1.l_orderkey
          and l3.l_suppkey <> l1.l_suppkey
          and l3.l_receiptdate > l3.l_commitdate
    )
  and s_nationkey = n_nationkey
  and n_name = 'SAUDI ARABIA'
group by
    s_name
order by
    numwait desc,
    s_name;